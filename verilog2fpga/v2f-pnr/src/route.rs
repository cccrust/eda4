use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;
use crate::arch::{ArchGraph, TileCoord};

#[derive(Debug, Clone)]
pub struct RoutingPath {
    pub tiles: Vec<TileCoord>,
}

#[derive(Debug, Clone)]
pub struct Routing {
    pub net_paths: Vec<RoutingPath>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct AStarNode {
    f: u64,
    g: u64,
    coord: TileCoord,
}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f.cmp(&self.f)
    }
}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn neighbors(coord: &TileCoord, arch: &ArchGraph) -> Vec<TileCoord> {
    let mut n = Vec::new();
    let rows = arch.ice40.num_rows();
    let cols = arch.ice40.num_cols();
    if coord.row > 0 { n.push(TileCoord { row: coord.row - 1, col: coord.col }); }
    if coord.row + 1 < rows { n.push(TileCoord { row: coord.row + 1, col: coord.col }); }
    if coord.col > 0 { n.push(TileCoord { row: coord.row, col: coord.col - 1 }); }
    if coord.col + 1 < cols { n.push(TileCoord { row: coord.row, col: coord.col + 1 }); }
    n
}

fn manhattan(a: &TileCoord, b: &TileCoord) -> u64 {
    let dr = if a.row > b.row { a.row - b.row } else { b.row - a.row };
    let dc = if a.col > b.col { a.col - b.col } else { b.col - a.col };
    (dr + dc) as u64
}

fn astar_route(source: &TileCoord, sinks: &[TileCoord], arch: &ArchGraph, congestion: &HashMap<TileCoord, u32>) -> Option<Vec<TileCoord>> {
    if sinks.is_empty() { return Some(vec![*source]); }
    let target = sinks[0];
    let mut open = BinaryHeap::new();
    let mut came_from: HashMap<TileCoord, TileCoord> = HashMap::new();
    let mut g_score: HashMap<TileCoord, u64> = HashMap::new();
    g_score.insert(*source, 0);
    open.push(AStarNode { f: manhattan(source, &target), g: 0, coord: *source });
    let mut visited = HashSet::new();
    while let Some(current) = open.pop() {
        if !visited.insert(current.coord) { continue; }
        if sinks.iter().any(|s| current.coord == *s) {
            let mut path = Vec::new();
            let mut pos = current.coord;
            path.push(pos);
            while let Some(&prev) = came_from.get(&pos) {
                path.push(prev);
                if prev == *source { break; }
                pos = prev;
            }
            path.reverse();
            return Some(path);
        }
        for n in neighbors(&current.coord, arch) {
            let cong_penalty = *congestion.get(&n).unwrap_or(&0) as u64;
            let tentative_g = g_score[&current.coord] + 1 + cong_penalty * 10;
            if tentative_g < *g_score.get(&n).unwrap_or(&u64::MAX) {
                g_score.insert(n, tentative_g);
                came_from.insert(n, current.coord);
                open.push(AStarNode { f: tentative_g + manhattan(&n, &target), g: tentative_g, coord: n });
            }
        }
    }
    None
}

pub fn route(placement: &crate::place::Placement, arch: &ArchGraph) -> Routing {
    let mut congestion: HashMap<TileCoord, u32> = HashMap::new();
    let mut net_paths = Vec::new();
    let max_iters = 5;
    for _iteration in 0..max_iters {
        net_paths.clear();
        let mut all_routed = true;
        for net in &placement.nets {
            let coords: Vec<TileCoord> = net.iter().map(|&i| placement.cell_to_coord[i].1).collect();
            if coords.len() < 2 {
                net_paths.push(RoutingPath { tiles: vec![coords[0]] });
                continue;
            }
            let source = coords[0];
            let sinks: Vec<TileCoord> = coords[1..].to_vec();
            match astar_route(&source, &sinks, arch, &congestion) {
                Some(path) => {
                    for t in &path {
                        *congestion.entry(*t).or_insert(0) += 1;
                    }
                    net_paths.push(RoutingPath { tiles: path });
                }
                None => {
                    all_routed = false;
                    net_paths.push(RoutingPath { tiles: vec![source] });
                }
            }
        }
        if all_routed { break; }
        for v in congestion.values_mut() { *v = v.saturating_sub(1); }
    }
    Routing { net_paths }
}
