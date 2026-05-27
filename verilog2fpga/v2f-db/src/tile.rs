/// iCE40 Tile 類型與拓撲資訊

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    Logic,
    Io,
    Bram,
    Dsp,
}

impl TileType {
    pub fn num_frames(&self) -> u32 {
        match self {
            TileType::Logic => 7,
            TileType::Io => 3,
            TileType::Bram => 14,
            TileType::Dsp => 7,
        }
    }
}

/// Tile 在 FPGA 中的位置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TilePos {
    pub row: u32,
    pub col: u32,
}

/// 某個 Tile 在 CRAM 中的 Frame 範圍
#[derive(Debug, Clone)]
pub struct TileFrameRange {
    pub pos: TilePos,
    pub tile_type: TileType,
    pub start_frame: u32,
    pub end_frame: u32,
}
