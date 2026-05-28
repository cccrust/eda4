use v2f_bitstream::cram::Cram;
use v2f_db::cram_addr::CramAddrMap;
use v2f_db::ice40::Ice40Device;
use v2f_db::tile::{TilePos, TileType};

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
/// 使用與 `CramAddrMap::tile_start_frame` 相同的 frame 計算，
/// 確保 decode 與 `apply_asc_to_cram` 的 encode 一致。
pub fn decode_cram(cram: &Cram, device: Ice40Device) -> Vec<DecodedTile> {
    let addr_map = CramAddrMap::new(device);
    let num_rows = device.num_rows();
    let num_cols = device.num_cols();
    let mut tiles = Vec::new();

    // 左邊 IO tile (col=0)
    for row in 0..num_rows {
        let pos = TilePos { row, col: 0 };
        let start = addr_map.tile_start_frame(&pos, TileType::Io);
        decode_one_tile(cram, &mut tiles, row, 0, "io", start, 3);
    }

    // 中間 Logic tiles (col=1..num_cols)
    for row in 0..num_rows {
        for col in 0..num_cols {
            let pos = TilePos { row, col };
            let start = addr_map.tile_start_frame(&pos, TileType::Logic);
            decode_one_tile(cram, &mut tiles, row, col + 1, "logic", start, 7);
        }
    }

    // 右邊 IO tile (col=num_cols+1)
    for row in 0..num_rows {
        let pos = TilePos {
            row,
            col: num_cols + 1,
        };
        let start = addr_map.tile_start_frame(&pos, TileType::Io);
        decode_one_tile(cram, &mut tiles, row, num_cols + 1, "io", start, 3);
    }

    tiles
}

fn decode_one_tile(
    cram: &Cram,
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
        let total_cols = dev.num_cols() + 2;
        assert_eq!(tiles.len() as u32, dev.num_rows() * total_cols);
        for t in &tiles {
            assert!(
                t.non_zero_words.is_empty(),
                "empty CRAM should have no non-zero words"
            );
        }
    }

    #[test]
    fn test_tile_type_counts() {
        let dev = Ice40Device::HX8K;
        let cram = Cram::new(dev);
        let tiles = decode_cram(&cram, dev);
        let io_count = tiles.iter().filter(|t| t.tile_type == "io").count();
        let logic_count = tiles.iter().filter(|t| t.tile_type == "logic").count();
        assert_eq!(io_count, dev.num_rows() as usize * 2);
        assert_eq!(
            logic_count,
            dev.num_rows() as usize * dev.num_cols() as usize
        );
    }
}
