use std::collections::HashSet;
use crate::arch::TileCoord;

pub struct PlacerCost {
    pub w_wire: f64,
    pub w_cong: f64,
}

impl Default for PlacerCost {
    fn default() -> Self {
        PlacerCost { w_wire: 1.0, w_cong: 0.5 }
    }
}

pub fn hpwl(net_bits: &[TileCoord]) -> f64 {
    if net_bits.is_empty() { return 0.0; }
    let min_row = net_bits.iter().map(|c| c.row).min().unwrap();
    let max_row = net_bits.iter().map(|c| c.row).max().unwrap();
    let min_col = net_bits.iter().map(|c| c.col).min().unwrap();
    let max_col = net_bits.iter().map(|c| c.col).max().unwrap();
    (max_row - min_row + max_col - min_col) as f64
}

pub fn total_wire_length(cell_to_coord: &[(String, TileCoord)], nets: &[Vec<usize>]) -> f64 {
    let mut total = 0.0;
    for net in nets {
        if net.len() < 2 { continue; }
        let mut min_row = u32::MAX;
        let mut max_row = 0u32;
        let mut min_col = u32::MAX;
        let mut max_col = 0u32;
        for &i in net {
            let c = &cell_to_coord[i].1;
            if c.row < min_row { min_row = c.row; }
            if c.row > max_row { max_row = c.row; }
            if c.col < min_col { min_col = c.col; }
            if c.col > max_col { max_col = c.col; }
        }
        total += (max_row - min_row + max_col - min_col) as f64;
    }
    total
}

pub fn congestion_penalty(cell_to_coord: &[(String, TileCoord)], _nets: &[Vec<usize>]) -> f64 {
    let mut used = HashSet::new();
    for (_, coord) in cell_to_coord {
        if !used.insert(*coord) {
            return 1000.0;
        }
    }
    0.0
}

pub fn evaluate(cell_to_coord: &[(String, TileCoord)], nets: &[Vec<usize>], params: &PlacerCost) -> f64 {
    let wire = total_wire_length(cell_to_coord, nets);
    let cong = congestion_penalty(cell_to_coord, nets);
    params.w_wire * wire + params.w_cong * cong
}
