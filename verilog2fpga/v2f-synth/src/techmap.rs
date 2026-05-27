use std::collections::HashMap;

use serde::Serialize;

use crate::netlist::*;

#[derive(Debug, Clone, Serialize)]
pub struct MappedCell {
    #[serde(rename = "type")]
    pub cell_type: String,
    pub parameters: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port_directions: Option<HashMap<String, String>>,
    pub connections: HashMap<String, Vec<u64>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct YosysModule {
    pub ports: HashMap<String, YosysPort>,
    pub cells: HashMap<String, MappedCell>,
    pub netnames: HashMap<String, YosysNet>,
}

#[derive(Debug, Clone, Serialize)]
pub struct YosysPort {
    pub direction: String,
    pub bits: Vec<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct YosysNet {
    pub bits: Vec<u64>,
    #[serde(skip_serializing_if = "is_zero")]
    pub hide_name: u8,
}

fn is_zero(n: &u8) -> bool { *n == 0 }

pub fn techmap_to_json(netlist: &Netlist) -> HashMap<String, YosysModule> {
    let mut ports = HashMap::new();
    let mut cells = HashMap::new();
    let mut netnames = HashMap::new();

    for p in &netlist.ports {
        ports.insert(p.name.clone(), YosysPort {
            direction: match p.direction {
                PortDir::Input => "input".to_string(),
                PortDir::Output => "output".to_string(),
                PortDir::Inout => "inout".to_string(),
            },
            bits: p.bits.clone(),
        });
    }

    for cell in &netlist.cells {
        let (cell_type, params, connections) = map_cell(cell);
        let dirs = get_port_directions(&cell_type);
        cells.insert(cell.name.clone(), MappedCell {
            cell_type,
            parameters: params,
            port_directions: dirs,
            connections,
        });
    }

    let mut bit_to_name: HashMap<u64, String> = HashMap::new();
    for b in &netlist.bits {
        if let Some(ref name) = b.name {
            bit_to_name.insert(b.id, name.clone());
        }
    }
    for p in &netlist.ports {
        for &b in &p.bits {
            bit_to_name.entry(b).or_insert_with(|| p.name.clone());
        }
    }

    let mut name_bits: HashMap<String, Vec<u64>> = HashMap::new();
    for (id, name) in bit_to_name {
        name_bits.entry(name).or_default().push(id);
    }
    for p in &netlist.ports {
        name_bits.insert(p.name.clone(), p.bits.clone());
    }

    for (name, bits) in &name_bits {
        let hide = if ports.contains_key(name) { 0 } else { 1 };
        netnames.insert(name.clone(), YosysNet { bits: bits.clone(), hide_name: hide });
    }

    let mut modules = HashMap::new();
    modules.insert(netlist.top.clone(), YosysModule { ports, cells, netnames });
    modules
}

fn map_cell(cell: &Cell) -> (String, HashMap<String, serde_json::Value>, HashMap<String, Vec<u64>>) {
    let mut params = HashMap::new();
    let mut conns = HashMap::new();

    match cell.kind {
        CellKind::Input => {
            conns.insert("Y".to_string(), cell.outputs[0].1.clone());
            ("$_INPUT_".to_string(), params, conns)
        }
        CellKind::Output => {
            conns.insert("A".to_string(), cell.inputs[0].1.clone());
            conns.insert("Y".to_string(), cell.outputs[0].1.clone());
            ("$_OUTPUT_".to_string(), params, conns)
        }
        CellKind::And => {
            conns.insert("A".to_string(), cell.inputs[0].1.clone());
            conns.insert("B".to_string(), cell.inputs[1].1.clone());
            conns.insert("Y".to_string(), cell.outputs[0].1.clone());
            ("$_AND_".to_string(), params, conns)
        }
        CellKind::Or => {
            conns.insert("A".to_string(), cell.inputs[0].1.clone());
            conns.insert("B".to_string(), cell.inputs[1].1.clone());
            conns.insert("Y".to_string(), cell.outputs[0].1.clone());
            ("$_OR_".to_string(), params, conns)
        }
        CellKind::Xor => {
            conns.insert("A".to_string(), cell.inputs[0].1.clone());
            conns.insert("B".to_string(), cell.inputs[1].1.clone());
            conns.insert("Y".to_string(), cell.outputs[0].1.clone());
            ("$_XOR_".to_string(), params, conns)
        }
        CellKind::Not => {
            conns.insert("A".to_string(), cell.inputs[0].1.clone());
            conns.insert("Y".to_string(), cell.outputs[0].1.clone());
            ("$_NOT_".to_string(), params, conns)
        }
        CellKind::Dff => {
            conns.insert("D".to_string(), cell.inputs[0].1.clone());
            conns.insert("Q".to_string(), cell.outputs[0].1.clone());
            ("$_DFF_P_".to_string(), params, conns)
        }
        CellKind::Const0 => {
            conns.insert("Y".to_string(), cell.outputs[0].1.clone());
            ("$_ZERO_".to_string(), params, conns)
        }
        CellKind::Const1 => {
            conns.insert("Y".to_string(), cell.outputs[0].1.clone());
            ("$_ONE_".to_string(), params, conns)
        }
        CellKind::Add | CellKind::Sub => {
            let (aidx, bidx) = if cell.inputs.len() >= 2 { (0, 1) } else { (0, 0) };
            conns.insert("A".to_string(), cell.inputs[aidx].1.clone());
            conns.insert("B".to_string(), cell.inputs[bidx].1.clone());
            conns.insert("Y".to_string(), cell.outputs[0].1.clone());
            let ty = if cell.kind == CellKind::Add { "$_ADD_" } else { "$_SUB_" };
            (ty.to_string(), params, conns)
        }
        CellKind::Mux2 => {
            conns.insert("A".to_string(), cell.inputs[0].1.clone());
            conns.insert("B".to_string(), cell.inputs[1].1.clone());
            conns.insert("S".to_string(), cell.inputs[2].1.clone());
            conns.insert("Y".to_string(), cell.outputs[0].1.clone());
            ("$_MUX_".to_string(), params, conns)
        }
        CellKind::Lut { init } => {
            for (i, (_name, bits)) in cell.inputs.iter().enumerate() {
                conns.insert(format!("I{i}"), bits.clone());
            }
            conns.insert("O".to_string(), cell.outputs[0].1.clone());
            params.insert("LUT_INIT".to_string(), serde_json::Value::Number(init.into()));
            ("ICESTORM_LC".to_string(), params, conns)
        }
        CellKind::Carry => {
            conns.insert("I0".to_string(), cell.inputs[0].1.clone());
            conns.insert("I1".to_string(), cell.inputs[1].1.clone());
            conns.insert("O".to_string(), cell.outputs[0].1.clone());
            ("$_CARRY_".to_string(), params, conns)
        }
    }
}

fn get_port_directions(cell_type: &str) -> Option<HashMap<String, String>> {
    let mut dirs = HashMap::new();
    match cell_type {
        "$_INPUT_" => { dirs.insert("Y".into(), "output".into()); Some(dirs) }
        "$_OUTPUT_" => { dirs.insert("A".into(), "input".into()); dirs.insert("Y".into(), "output".into()); Some(dirs) }
        "$_AND_" | "$_OR_" | "$_XOR_" | "$_ADD_" | "$_SUB_" => {
            dirs.insert("A".into(), "input".into());
            dirs.insert("B".into(), "input".into());
            dirs.insert("Y".into(), "output".into());
            Some(dirs)
        }
        "$_NOT_" => {
            dirs.insert("A".into(), "input".into());
            dirs.insert("Y".into(), "output".into());
            Some(dirs)
        }
        "$_DFF_P_" => {
            dirs.insert("D".into(), "input".into());
            dirs.insert("C".into(), "input".into());
            dirs.insert("Q".into(), "output".into());
            Some(dirs)
        }
        "$_ZERO_" | "$_ONE_" => {
            dirs.insert("Y".into(), "output".into());
            Some(dirs)
        }
        "$_MUX_" => {
            dirs.insert("A".into(), "input".into());
            dirs.insert("B".into(), "input".into());
            dirs.insert("S".into(), "input".into());
            dirs.insert("Y".into(), "output".into());
            Some(dirs)
        }
        "ICESTORM_LC" => {
            dirs.insert("I0".into(), "input".into());
            dirs.insert("I1".into(), "input".into());
            dirs.insert("I2".into(), "input".into());
            dirs.insert("I3".into(), "input".into());
            dirs.insert("O".into(), "output".into());
            Some(dirs)
        }
        _ => None,
    }
}
