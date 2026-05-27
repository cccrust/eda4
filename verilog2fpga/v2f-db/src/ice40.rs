/// iCE40 系列裝置參數
///
/// 參考 Project IceStorm (icestorm) 的 CRAM 規格：
/// - 每個 Frame = 33 words × 40 bits = 1320 bits = 165 bytes
/// - 每個 Logic Tile = 7 Frames
/// - 每個 IO Tile (左/右邊緣) = 特定 Frame 數量
///
/// 總 Frame 數計算方式：
///   每一列 Frame 數 = (logic_cols × 7) + io_left_frames + io_right_frames
///   總 Frame 數 = rows × frames_per_row

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ice40Device {
    HX1K,
    HX4K,
    HX8K,
    LP1K,
    UP5K,
}

impl Ice40Device {
    pub const BITS_PER_FRAME: u32 = 1320;
    pub const WORDS_PER_FRAME: u32 = 33;
    pub const BITS_PER_WORD: u32 = 40;
    pub const FRAME_BYTES: u32 = 165;

    pub fn name(&self) -> &'static str {
        match self {
            Ice40Device::HX1K => "hx1k",
            Ice40Device::HX4K => "hx4k",
            Ice40Device::HX8K => "hx8k",
            Ice40Device::LP1K => "lp1k",
            Ice40Device::UP5K => "up5k",
        }
    }

    /// 邏輯列數
    pub fn num_rows(&self) -> u32 {
        match self {
            Ice40Device::HX1K => 30,
            Ice40Device::HX4K => 40,
            Ice40Device::HX8K => 70,
            Ice40Device::LP1K => 30,
            Ice40Device::UP5K => 40,
        }
    }

    /// 邏輯行數
    pub fn num_cols(&self) -> u32 {
        match self {
            Ice40Device::HX1K => 16,
            Ice40Device::HX4K => 20,
            Ice40Device::HX8K => 28,
            Ice40Device::LP1K => 16,
            Ice40Device::UP5K => 22,
        }
    }

    /// 每列 Frame 數 = logic_cols × 7 (每個 logic tile) + IO 邊界
    pub fn frames_per_row(&self) -> u32 {
        let logic = self.num_cols() * 7;
        match self {
            Ice40Device::HX1K => logic + 3 + 3,
            Ice40Device::HX4K => logic + 3 + 3,
            Ice40Device::HX8K => logic + 3 + 3,
            Ice40Device::LP1K => logic + 3 + 3,
            Ice40Device::UP5K => logic + 3 + 3,
        }
    }

    /// 總 Frame 數
    pub fn total_frames(&self) -> u32 {
        self.num_rows() * self.frames_per_row()
    }
}
