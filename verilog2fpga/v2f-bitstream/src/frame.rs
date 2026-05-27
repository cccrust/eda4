/// iCE40 Frame：1320 bits = 165 bytes
///
/// 每個 Frame 包含 33 個 Word，每個 Word 40 bits。
/// 儲存為位元組陣列，提供逐位元存取。

pub const FRAME_BITS: usize = 1320;
pub const FRAME_BYTES: usize = 165;
pub const WORDS_PER_FRAME: usize = 33;
pub const BITS_PER_WORD: usize = 40;

#[derive(Debug, Clone)]
pub struct Frame {
    data: [u8; FRAME_BYTES],
}

impl Frame {
    pub fn new() -> Self {
        Frame {
            data: [0u8; FRAME_BYTES],
        }
    }

    /// 從原始位元組還原
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut data = [0u8; FRAME_BYTES];
        let len = bytes.len().min(FRAME_BYTES);
        data[..len].copy_from_slice(&bytes[..len]);
        Frame { data }
    }

    pub fn as_bytes(&self) -> &[u8; FRAME_BYTES] {
        &self.data
    }

    pub fn into_bytes(self) -> [u8; FRAME_BYTES] {
        self.data
    }

    /// 設定特定位元 (0-indexed, bit 0 = LSB of byte 0)
    pub fn set_bit(&mut self, bit_pos: usize) {
        assert!(bit_pos < FRAME_BITS, "bit_pos {bit_pos} >= {FRAME_BITS}");
        let byte = bit_pos / 8;
        let bit = bit_pos % 8;
        self.data[byte] |= 1 << bit;
    }

    /// 清除特定位元
    pub fn clear_bit(&mut self, bit_pos: usize) {
        assert!(bit_pos < FRAME_BITS, "bit_pos {bit_pos} >= {FRAME_BITS}");
        let byte = bit_pos / 8;
        let bit = bit_pos % 8;
        self.data[byte] &= !(1 << bit);
    }

    /// 讀取特定位元
    pub fn get_bit(&self, bit_pos: usize) -> bool {
        assert!(bit_pos < FRAME_BITS, "bit_pos {bit_pos} >= {FRAME_BITS}");
        let byte = bit_pos / 8;
        let bit = bit_pos % 8;
        (self.data[byte] >> bit) & 1 == 1
    }

    /// 寫入 Word (Word 編號 0..32, 每個 40 bits, little-endian)
    pub fn set_word(&mut self, word_idx: usize, value: u64) {
        assert!(word_idx < WORDS_PER_FRAME);
        let mask = (1u64 << BITS_PER_WORD) - 1;
        let v = value & mask;
        let base = word_idx * 5;
        self.data[base] = v as u8;
        self.data[base + 1] = (v >> 8) as u8;
        self.data[base + 2] = (v >> 16) as u8;
        self.data[base + 3] = (v >> 24) as u8;
        self.data[base + 4] = (v >> 32) as u8;
    }

    /// 讀取 Word
    pub fn get_word(&self, word_idx: usize) -> u64 {
        assert!(word_idx < WORDS_PER_FRAME);
        let base = word_idx * 5;
        let mut v = 0u64;
        v |= self.data[base] as u64;
        v |= (self.data[base + 1] as u64) << 8;
        v |= (self.data[base + 2] as u64) << 16;
        v |= (self.data[base + 3] as u64) << 24;
        v |= (self.data[base + 4] as u64) << 32;
        v
    }
}

impl Default for Frame {
    fn default() -> Self {
        Frame::new()
    }
}
