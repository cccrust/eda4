use std::collections::HashMap;
use crate::hdl::{HdlExpr, HdlModule, HdlPortDir, HdlStmt};
use serde::Serialize;

#[derive(Serialize)]
pub struct YosysPort {
    pub direction: String,
    pub bits: Vec<u64>,
}

#[derive(Serialize)]
pub struct YosysCell {
    #[serde(rename = "type")]
    pub cell_type: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub port_directions: HashMap<String, String>,
    pub connections: HashMap<String, Vec<u64>>,
}

#[derive(Serialize)]
pub struct YosysNet {
    pub bits: Vec<u64>,
    pub hide_name: u8,
}

#[derive(Serialize)]
pub struct YosysModule {
    pub ports: HashMap<String, YosysPort>,
    pub cells: HashMap<String, YosysCell>,
    pub netnames: HashMap<String, YosysNet>,
}

#[derive(Serialize)]
pub struct SynthOutput {
    pub creator: String,
    pub modules: HashMap<String, YosysModule>,
}

pub fn compile(module: &HdlModule) -> String {
    let mut next_bit: u64 = 1;
    let mut bit_for: HashMap<String, Vec<u64>> = HashMap::new();
    let mut cells: HashMap<String, YosysCell> = HashMap::new();
    let mut netnames: HashMap<String, YosysNet> = HashMap::new();
    let mut ports_json: HashMap<String, YosysPort> = HashMap::new();

    for p in &module.ports {
        let bits: Vec<u64> = (0..p.width).map(|i| { let b = next_bit; next_bit += 1; b }).collect();
        let name = p.name.clone();
        bit_for.insert(format!("port_{}", name), bits.clone());
        ports_json.insert(name.clone(), YosysPort {
            direction: match p.direction {
                HdlPortDir::Input => "input".into(),
                HdlPortDir::Output => "output".into(),
                HdlPortDir::Inout => "inout".into(),
            },
            bits: bits.clone(),
        });
    }

    let mut cell_idx = 0u64;
    for stmt in &module.stmts {
        match stmt {
            HdlStmt::DeclReg { name, width } | HdlStmt::DeclWire { name, width } => {
                if !bit_for.contains_key(name) {
                    let bits: Vec<u64> = (0..*width).map(|_| { let b = next_bit; next_bit += 1; b }).collect();
                    bit_for.insert(name.clone(), bits);
                }
            }
            HdlStmt::Assign { target, value } | HdlStmt::Nonblocking { target, value } => {
                let target_bits = bit_for.get(target).cloned().unwrap_or_else(|| {
                    let bits: Vec<u64> = (0..1).map(|_| { let b = next_bit; next_bit += 1; b }).collect();
                    bit_for.insert(target.clone(), bits.clone());
                    bits
                });
                let value_bits = eval_expr(value, &mut bit_for, &mut next_bit);
                let stmt_is_dff = matches!(stmt, HdlStmt::Nonblocking { .. });
                if stmt_is_dff {
                    for (i, &tb) in target_bits.iter().enumerate() {
                        let vb = *value_bits.get(i).unwrap_or(&0);
                        cell_idx += 1;
                        let cell_name = format!("${}", cell_idx);
                        let mut conns = HashMap::new();
                        conns.insert("D".into(), vec![vb]);
                        conns.insert("Q".into(), vec![tb]);
                        cells.insert(cell_name.clone(), YosysCell {
                            cell_type: "$_DFF_P_".into(),
                            parameters: HashMap::new(),
                            port_directions: {
                                let mut d = HashMap::new();
                                d.insert("D".into(), "input".into());
                                d.insert("C".into(), "input".into());
                                d.insert("Q".into(), "output".into());
                                d
                            },
                            connections: conns,
                        });
                    }
                } else {
                    for (i, &tb) in target_bits.iter().enumerate() {
                        let vb = *value_bits.get(i).unwrap_or(&0);
                        cell_idx += 1;
                        let cell_name = format!("${}", cell_idx);
                        let mut conns = HashMap::new();
                        conns.insert("A".into(), vec![vb]);
                        conns.insert("Y".into(), vec![tb]);
                        cells.insert(cell_name.clone(), YosysCell {
                            cell_type: "$_OUTPUT_".into(),
                            parameters: HashMap::new(),
                            port_directions: {
                                let mut d = HashMap::new();
                                d.insert("A".into(), "input".into());
                                d.insert("Y".into(), "output".into());
                                d
                            },
                            connections: conns,
                        });
                    }
                }
            }
            HdlStmt::Blocking { target, value } => {
                let target_bits = bit_for.get(target).cloned().unwrap_or_default();
                let value_bits = eval_expr(value, &mut bit_for, &mut next_bit);
                for (i, &tb) in target_bits.iter().enumerate() {
                    let vb = *value_bits.get(i).unwrap_or(&0);
                    cell_idx += 1;
                    let mut conns = HashMap::new();
                    conns.insert("A".into(), vec![vb]);
                    conns.insert("Y".into(), vec![tb]);
                    cells.insert(format!("${}", cell_idx), YosysCell {
                        cell_type: "$_OUTPUT_".into(),
                        parameters: HashMap::new(),
                        port_directions: {
                            let mut d = HashMap::new();
                            d.insert("A".into(), "input".into());
                            d.insert("Y".into(), "output".into());
                            d
                        },
                        connections: conns,
                    });
                }
            }
        }
    }

    for (name, bits) in &bit_for {
        let hide = if ports_json.contains_key(name) { 0 } else { 1 };
        netnames.insert(name.clone(), YosysNet { bits: bits.clone(), hide_name: hide });
    }

    let mut modules = HashMap::new();
    modules.insert(module.name.clone(), YosysModule {
        ports: ports_json,
        cells,
        netnames,
    });

    let output = SynthOutput {
        creator: "v2f-rust v0.5".into(),
        modules,
    };

    serde_json::to_string_pretty(&output).unwrap()
}

fn eval_expr(
    expr: &HdlExpr,
    bit_for: &mut HashMap<String, Vec<u64>>,
    next_bit: &mut u64,
) -> Vec<u64> {
    match expr {
        HdlExpr::Const(v, w) => {
            let width = if *w > 0 { *w } else { 1u32.max(64 - v.leading_zeros()) };
            let mut bits = Vec::new();
            for i in 0..width {
                let b = *next_bit; *next_bit += 1;
                bits.push(b);
            }
            bits
        }
        HdlExpr::Ident(name) => {
            bit_for.get(name).cloned().unwrap_or_default()
        }
        HdlExpr::Index(base, idx) => {
            let base_bits = eval_expr(base, bit_for, next_bit);
            if (*idx as usize) < base_bits.len() {
                vec![base_bits[*idx as usize]]
            } else { vec![] }
        }
        HdlExpr::Range(base, msb, lsb) => {
            let base_bits = eval_expr(base, bit_for, next_bit);
            let (lo, hi) = if msb >= lsb { (*lsb as usize, *msb as usize) } else { (*msb as usize, *lsb as usize) };
            if hi < base_bits.len() {
                base_bits[lo..=hi].to_vec()
            } else { vec![] }
        }
        HdlExpr::Concat(exprs) => {
            let mut bits = Vec::new();
            for e in exprs { bits.extend(eval_expr(e, bit_for, next_bit)); }
            bits
        }
        _ => {
            let b = *next_bit; *next_bit += 1;
            vec![b]
        }
    }
}
