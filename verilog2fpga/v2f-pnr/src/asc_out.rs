use std::fmt::Write;
use crate::arch::ArchGraph;
use crate::place::Placement;
use crate::route::Routing;

pub fn write_asc(placement: &Placement, routing: &Routing, arch: &ArchGraph) -> String {
    let mut s = String::new();
    writeln!(s, ".device {}", arch_device_name(arch)).ok();
    writeln!(s).ok();
    for (name, coord) in &placement.cell_to_coord {
        writeln!(s, ".logic_tile {} {}", coord.col, coord.row).ok();
        writeln!(s, "  .sym {} 0 0 0 0 \"{}\"", coord.row * 100 + coord.col, name).ok();
    }
    writeln!(s).ok();
    for (i, path) in routing.net_paths.iter().enumerate() {
        for w in path.tiles.windows(2) {
            let (r1, c1) = (w[0].row, w[0].col);
            let (r2, c2) = (w[1].row, w[1].col);
            let track = i % 8;
            let from_col = if c1 < c2 || r1 < r2 { c1 } else { c2 };
            let to_col = if c1 > c2 || r1 > r2 { c1 } else { c2 };
            writeln!(s, ".wiring {} {} {} {}", from_col, r1.min(r2), to_col, track).ok();
        }
    }
    s
}

fn arch_device_name(arch: &ArchGraph) -> &str {
    match arch.device {
        v2f_core::Device::HX1K => "HX1K-TQ144",
        v2f_core::Device::HX4K => "HX4K-TQ144",
        v2f_core::Device::HX8K => "HX8K-CT256",
        v2f_core::Device::LP1K => "LP1K-CM36",
        v2f_core::Device::UP5K => "UP5K-SG48",
    }
}
