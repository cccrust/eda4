use serde::Deserialize;
use std::collections::HashMap;

use crate::arch::ArchGraph;
use crate::asc_out;
use crate::place;
use crate::route;

#[derive(Debug, Deserialize)]
pub struct SynthJson {
    pub creator: Option<String>,
    pub modules: HashMap<String, ModuleJson>,
}

#[derive(Debug, Deserialize)]
pub struct ModuleJson {
    pub ports: HashMap<String, PortJson>,
    pub cells: HashMap<String, CellJson>,
    pub netnames: HashMap<String, NetJson>,
}

#[derive(Debug, Deserialize)]
pub struct PortJson {
    pub direction: String,
    pub bits: Vec<u64>,
}

#[derive(Debug, Deserialize)]
pub struct CellJson {
    #[serde(rename = "type")]
    pub cell_type: String,
    pub parameters: Option<HashMap<String, serde_json::Value>>,
    pub port_directions: Option<HashMap<String, String>>,
    pub connections: HashMap<String, Vec<u64>>,
}

#[derive(Debug, Deserialize)]
pub struct NetJson {
    pub bits: Vec<u64>,
    pub hide_name: Option<u8>,
}

pub struct PnrNetlist {
    pub cell_names: Vec<String>,
    pub cell_types: Vec<String>,
    pub net_conns: Vec<Vec<usize>>,
}

pub fn parse_json(json_str: &str) -> PnrNetlist {
    let parsed: SynthJson = serde_json::from_str(json_str).expect("無效的 JSON");
    let top_mod = parsed.modules.values().next().expect("無模組");
    let mut cell_names = Vec::new();
    let mut cell_types = Vec::new();
    let mut bit_to_cell: HashMap<u64, usize> = HashMap::new();
    for (name, cell) in &top_mod.cells {
        let idx = cell_names.len();
        cell_names.push(name.clone());
        cell_types.push(cell.cell_type.clone());
        for conns in cell.connections.values() {
            for &b in conns {
                bit_to_cell.entry(b).or_insert(idx);
            }
        }
    }
    for (name, port) in &top_mod.ports {
        let idx = cell_names.len();
        cell_names.push(format!("port_{}", name));
        cell_types.push("PORT".to_string());
        for &b in &port.bits {
            bit_to_cell.entry(b).or_insert(idx);
        }
    }
    let mut net_conns: Vec<Vec<usize>> = Vec::new();
    let mut processed = std::collections::HashSet::new();
    for (&_bit, &cell_idx) in &bit_to_cell {
        if processed.contains(&cell_idx) { continue; }
        processed.insert(cell_idx);
        let mut group = vec![cell_idx];
        for (b, &ci) in &bit_to_cell {
            if ci != cell_idx {
                let same_net = top_mod.netnames.values().any(|n| n.bits.contains(b) && n.bits.contains(&_bit));
                if same_net {
                    if processed.insert(ci) {
                        group.push(ci);
                    }
                }
            }
        }
        net_conns.push(group);
    }
    PnrNetlist { cell_names, cell_types, net_conns }
}

pub fn run_pnr(json_str: &str, device: v2f_core::Device) -> String {
    let netlist = parse_json(json_str);
    let arch = ArchGraph::new(device);
    let mut placement = place::random_placement(&netlist.cell_names, &netlist.net_conns, &arch);
    place::place(&mut placement, &arch);
    let routing = route::route(&placement, &arch);
    asc_out::write_asc(&placement, &routing, &arch)
}
