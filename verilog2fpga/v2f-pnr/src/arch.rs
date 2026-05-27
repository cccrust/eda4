use v2f_core::Device;
use v2f_db::tile::TileType;
use v2f_db::Ice40Device;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileCoord {
    pub row: u32,
    pub col: u32,
}

#[derive(Debug, Clone)]
pub struct ArchTile {
    pub coord: TileCoord,
    pub tile_type: TileType,
}

#[derive(Debug, Clone)]
pub struct ArchGraph {
    pub device: Device,
    pub ice40: Ice40Device,
    pub tiles: Vec<ArchTile>,
    pub logic_rows: u32,
    pub logic_cols: u32,
}

impl ArchGraph {
    pub fn new(device: Device) -> Self {
        let ice40 = match device {
            Device::HX1K => Ice40Device::HX1K,
            Device::HX4K => Ice40Device::HX4K,
            Device::HX8K => Ice40Device::HX8K,
            Device::LP1K => Ice40Device::LP1K,
            Device::UP5K => Ice40Device::UP5K,
        };
        let total_rows = ice40.num_rows();
        let total_cols = ice40.num_cols();
        let logic_rows = total_rows.saturating_sub(2);
        let logic_cols = total_cols;
        let mut tiles = Vec::new();
        for r in 0..total_rows {
            for c in 0..total_cols {
                let tile_type = if r == 0 || r == total_rows - 1 {
                    TileType::Io
                } else if c == 0 || c == total_cols - 1 {
                    TileType::Io
                } else {
                    TileType::Logic
                };
                tiles.push(ArchTile {
                    coord: TileCoord { row: r, col: c },
                    tile_type,
                });
            }
        }
        ArchGraph { device, ice40, tiles, logic_rows, logic_cols }
    }

    pub fn logic_tiles(&self) -> Vec<TileCoord> {
        self.tiles.iter()
            .filter(|t| t.tile_type == TileType::Logic)
            .map(|t| t.coord)
            .collect()
    }

    pub fn io_tiles(&self) -> Vec<TileCoord> {
        self.tiles.iter()
            .filter(|t| t.tile_type == TileType::Io)
            .map(|t| t.coord)
            .collect()
    }

    pub fn is_valid_placement(&self, coord: TileCoord, cell_type: &str) -> bool {
        let tile = match self.tiles.iter().find(|t| t.coord == coord) {
            Some(t) => t,
            None => return false,
        };
        match tile.tile_type {
            TileType::Logic => {
                !cell_type.starts_with("SB_")
            }
            TileType::Io => {
                cell_type == "SB_IO" || cell_type == "SB_GB_IO"
                    || cell_type.starts_with("$_INPUT_")
                    || cell_type.starts_with("$_OUTPUT_")
            }
            _ => false,
        }
    }
}
