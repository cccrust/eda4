use crate::ice40::Ice40Device;
use crate::tile::{TilePos, TileType};

/// CRAM 位址：定位到某個 Frame 中的某個 Word 的某個 Bit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CramAddr {
    pub frame: u32,
    pub word: u32,
    pub bit: u32,
}

/// Frame 位址計算模組
///
/// 將 (row, col, bit_type) 映射到絕對 Frame 編號。
/// iCE40 CRAM 的 Frame 排列方式為 Row-major：
///   每列從左到右掃描所有 Tile，每個 Tile 貢獻 N 個 Frame。
pub struct CramAddrMap {
    device: Ice40Device,
}

impl CramAddrMap {
    pub fn new(device: Ice40Device) -> Self {
        CramAddrMap { device }
    }

    /// 計算某個 Tile 的起始 Frame 編號
    pub fn tile_start_frame(&self, pos: &TilePos, tile_type: TileType) -> u32 {
        let frames_per_row = self.device.frames_per_row();
        let row_offset = pos.row * frames_per_row;
        let col_offset = pos.col * tile_type.num_frames();
        row_offset + col_offset
    }

    /// 計算某個 Tile 內某個 Bit 所在的絕對 Frame / Word / Bit
    pub fn resolve(
        &self,
        pos: &TilePos,
        tile_type: TileType,
        frame_within_tile: u32,
        word: u32,
        bit: u32,
    ) -> CramAddr {
        let start = self.tile_start_frame(pos, tile_type);
        CramAddr {
            frame: start + frame_within_tile,
            word,
            bit,
        }
    }

    /// 所有 Frame 編號（0..total_frames）
    pub fn all_frames(&self) -> impl Iterator<Item = u32> {
        0..self.device.total_frames()
    }
}
