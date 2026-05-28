use v2f_bitstream::cram::Cram;
use v2f_db::ice40::Ice40Device;

/// 單個 tile 的解碼結果
#[derive(Debug, Clone)]
pub struct DecodedTile {
    pub row: u32,
    pub col: u32,
    pub tile_type: &'static str,
    /// 每一筆 = (frame_within_tile, word_idx, word_value)
    pub non_zero_words: Vec<(u32, u32, u64)>,
}

/// 將 CRAM 解碼為每個 tile 的 raw frame 資料
///
/// CRAM frame 排列 (每列):
///   [IO left × 3] [Logic col 0 × 7] ... [Logic col N-1 × 7] [IO right × 3]
pub fn decode_cram(cram: &Cram, device: Ice40Device) -> Vec<DecodedTile> {
    let fpr = device.frames_per_row();
    let num_rows = device.num_rows();
    let num_cols = device.num_cols();
    let mut tiles = Vec::new();

    // 左邊 IO tile (col=0, 每列 3 frames)
    for row in 0..num_rows {
        let start = row * fpr;
        decode_one_tile(
            cram,
            device,
            &mut tiles,
            row,
            0,
            "io",
            start,
            3,
        );
    }

    // 中間 Logic tiles (col=1..num_cols, 每個 7 frames)
    for row in 0..num_rows {
        for col in 0..num_cols {
            let start = row * fpr + 3 + col * 7;
            decode_one_tile(
                cram,
                device,
                &mut tiles,
                row,
                col + 1,
                "logic",
                start,
                7,
            );
        }
    }

    // 右邊 IO tile (col=num_cols+1, 每列 3 frames)
    for row in 0..num_rows {
        let start = row * fpr + fpr - 3;
        decode_one_tile(
            cram,
            device,
            &mut tiles,
            row,
            num_cols + 1,
            "io",
            start,
            3,
        );
    }

    tiles
}

fn decode_one_tile(
    cram: &Cram,
    _device: Ice40Device,
    tiles: &mut Vec<DecodedTile>,
    row: u32,
    col: u32,
    tile_type: &'static str,
    start_frame: u32,
    num_frames: u32,
) {
    let mut non_zero_words = Vec::new();
    for fi in 0..num_frames {
        let frame = cram.get_frame(start_frame + fi);
        for wi in 0..33u32 {
            let word = frame.get_word(wi as usize);
            if word != 0 {
                non_zero_words.push((fi, wi, word));
            }
        }
    }
    tiles.push(DecodedTile {
        row,
        col,
        tile_type,
        non_zero_words,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_empty_cram_hx1k() {
        let dev = Ice40Device::HX1K;
        let cram = Cram::new(dev);
        let tiles = decode_cram(&cram, dev);
        let total = dev.num_rows() * (dev.num_cols() + 2);
        assert_eq!(tiles.len() as u32, total);
        for t in &tiles {
            assert!(t.non_zero_words.is_empty(), "empty CRAM should have no non-zero words");
        }
    }

    #[test]
    fn test_tile_type_counts() {
        let dev = Ice40Device::HX8K;
        let cram = Cram::new(dev);
        let tiles = decode_cram(&cram, dev);
        let io_count = tiles.iter().filter(|t| t.tile_type == "io").count();
        let logic_count = tiles.iter().filter(|t| t.tile_type == "logic").count();
        // HX8K: 70 rows × 2 IO cols = 140 IO tiles, 70 × 28 = 1960 logic tiles
        assert_eq!(io_count, dev.num_rows() as usize * 2);
        assert_eq!(logic_count, dev.num_rows() as usize * dev.num_cols() as usize);
    }
}
