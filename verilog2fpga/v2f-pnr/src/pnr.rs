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
    // Collect all cell names + types
    for (name, cell) in &top_mod.cells {
        cell_names.push(name.clone());
        cell_types.push(cell.cell_type.clone());
    }
    // Add ports as special "PORT" cells
    for (name, _port) in &top_mod.ports {
        cell_names.push(format!("port_{}", name));
        cell_types.push("PORT".to_string());
    }
    // Build bit → cell indices lookup
    let mut bit_to_cells: HashMap<u64, Vec<usize>> = HashMap::new();
    for (i, (name, cell)) in top_mod.cells.iter().enumerate() {
        for conns in cell.connections.values() {
            for &b in conns {
                bit_to_cells.entry(b).or_default().push(i);
            }
        }
    }
    for (i, (name, _port)) in top_mod.ports.iter().enumerate() {
        let idx = cell_names.len() - top_mod.ports.len() + i;
        for &b in &_port.bits {
            bit_to_cells.entry(b).or_default().push(idx);
        }
    }
    // Build net groups from netnames: each netname = one group of connected cells
    let mut net_conns: Vec<Vec<usize>> = Vec::new();
    let mut processed: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for (_net_name, net) in &top_mod.netnames {
        let mut group: Vec<usize> = Vec::new();
        for &b in &net.bits {
            if let Some(indices) = bit_to_cells.get(&b) {
                for &idx in indices {
                    if processed.insert(idx) {
                        group.push(idx);
                    }
                }
            }
        }
        if !group.is_empty() {
            net_conns.push(group);
        }
    }
    // Add any unconnected cells as single-cell nets
    for idx in 0..cell_names.len() {
        if !processed.contains(&idx) {
            net_conns.push(vec![idx]);
        }
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
