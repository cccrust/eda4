use rand::Rng;
use crate::arch::{ArchGraph, TileCoord};
use crate::cost::{self, PlacerCost};

pub struct Placement {
    pub cell_to_coord: Vec<(String, TileCoord)>,
    pub nets: Vec<Vec<usize>>,
    pub cost: f64,
}

pub fn build_nets(_cell_names: &[String], conns: &[Vec<usize>]) -> Vec<Vec<usize>> {
    conns.to_vec()
}

pub fn random_placement(cell_names: &[String], conns: &[Vec<usize>], arch: &ArchGraph) -> Placement {
    let logic_tiles = arch.logic_tiles();
    let mut rng = rand::thread_rng();
    let cell_to_coord: Vec<(String, TileCoord)> = cell_names.iter().map(|name| {
        let idx = rng.gen_range(0..logic_tiles.len());
        (name.clone(), logic_tiles[idx])
    }).collect();
    let nets = build_nets(cell_names, conns);
    let cost = cost::evaluate(&cell_to_coord, &nets, &PlacerCost::default());
    Placement { cell_to_coord, nets, cost }
}

pub fn place(placement: &mut Placement, arch: &ArchGraph) {
    let mut rng = rand::thread_rng();
    let logic_tiles = arch.logic_tiles();
    let cost_params = PlacerCost::default();
    let num_cells = placement.cell_to_coord.len();
    let t_start = 100.0;
    let t_end = 1.0;
    let cooling = 0.9;
    let iters_per_step = (num_cells * 50).max(100);
    let mut temp = t_start;
    let mut stall = 0;
    while temp > t_end && stall < 5 {
        let mut improved = false;
        for _ in 0..iters_per_step {
            let i = rng.gen_range(0..num_cells);
            let old = placement.cell_to_coord[i].1;
            let new_coord = logic_tiles[rng.gen_range(0..logic_tiles.len())];
            placement.cell_to_coord[i].1 = new_coord;
            let new_cost = cost::evaluate(&placement.cell_to_coord, &placement.nets, &cost_params);
            let delta = new_cost - placement.cost;
            if delta < 0.0 || rng.gen::<f64>() < (-delta / temp).exp() {
                placement.cost = new_cost;
                improved = true;
            } else {
                placement.cell_to_coord[i].1 = old;
            }
        }
        if !improved { stall += 1; } else { stall = 0; }
        temp *= cooling;
    }
}
