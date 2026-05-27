use v2f_db::ice40::Ice40Device;

use crate::frame::Frame;

/// CRAM (Configuration RAM) 記憶體模型
///
/// 儲存所有 Frame 的完整狀態。每個 Frame 為 1320 bits (165 bytes)。
#[derive(Debug, Clone)]
pub struct Cram {
    pub device: Ice40Device,
    frames: Vec<Frame>,
}

impl Cram {
    /// 建立空白 CRAM（全 0）
    pub fn new(device: Ice40Device) -> Self {
        let n = device.total_frames() as usize;
        let frames = (0..n).map(|_| Frame::new()).collect();
        Cram { device, frames }
    }

    /// 取得特定 Frame 的可變參考
    pub fn get_frame_mut(&mut self, frame_idx: u32) -> &mut Frame {
        assert!(
            (frame_idx as usize) < self.frames.len(),
            "frame_idx {frame_idx} >= {}",
            self.frames.len()
        );
        &mut self.frames[frame_idx as usize]
    }

    /// 取得特定 Frame 的參考
    pub fn get_frame(&self, frame_idx: u32) -> &Frame {
        assert!(
            (frame_idx as usize) < self.frames.len(),
            "frame_idx {frame_idx} >= {}",
            self.frames.len()
        );
        &self.frames[frame_idx as usize]
    }

    /// 所有 Frame 迭代器
    pub fn frames(&self) -> impl Iterator<Item = &Frame> {
        self.frames.iter()
    }

    /// Frame 總數
    pub fn num_frames(&self) -> u32 {
        self.frames.len() as u32
    }

    /// 將 CRAM 序列化為位元組向量（僅 CRAM 資料，不含前導/CRC）
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.frames.len() * 165);
        for frame in &self.frames {
            buf.extend_from_slice(frame.as_bytes());
        }
        buf
    }
}
