use v2f_bitstream::cram::Cram;
use v2f_bitstream::frame::{FRAME_BITS, BITS_PER_WORD};
use v2f_db::cram_addr::CramAddrMap;
use v2f_db::ice40::Ice40Device;
use v2f_db::tile::{TilePos, TileType};

/// Decoded wiring entry (reverse of apply_asc_to_cram encoding)
#[derive(Debug, Clone)]
pub struct DecodedWiring {
    pub bit_index: u32,
}

/// 單個 tile 的解碼結果
#[derive(Debug, Clone)]
pub struct DecodedTile {
    pub row: u32,
    pub col: u32,
    pub tile_type: &'static str,
    /// 每一筆 = (frame_within_tile, word_idx, word_value)
    pub non_zero_words: Vec<(u32, u32, u64)>,
    /// Decoded wiring bit indices
    pub wiring: Vec<DecodedWiring>,
}

/// Decode synckey from the last CRAM frame
pub fn decode_synckey(cram: &Cram) -> u32 {
    let last_frame = cram.get_frame(cram.num_frames() - 1);
    let fb = FRAME_BITS;
    let mut key = 0u32;
    for i in 0..4 {
        let mut byte = 0u8;
        for bit in 0..8 {
            let pos = (fb - 1) - (i * 8 + bit);
            if last_frame.get_bit(pos) {
                byte |= 1 << bit;
            }
        }
        key |= (byte as u32) << (i * 8);
    }
    key
}

/// 將 CRAM 解碼為每個 tile 的 raw frame 資料 + 可解碼的 wiring
///
/// 使用與 `CramAddrMap::tile_start_frame` 相同的 frame 計算，
/// 確保 decode 與 `apply_asc_to_cram` 的 encode 一致。
pub fn decode_cram(cram: &Cram, device: Ice40Device) -> Vec<DecodedTile> {
    let addr_map = CramAddrMap::new(device);
    let num_rows = device.num_rows();
    let num_cols = device.num_cols();
    let mut tiles = Vec::new();

    for row in 0..num_rows {
        let pos = TilePos { row, col: 0 };
        let start = addr_map.tile_start_frame(&pos, TileType::Io);
        decode_one_tile(cram, &mut tiles, row, 0, "io", start, 3, 440);
    }

    for row in 0..num_rows {
        for col in 0..num_cols {
            let pos = TilePos { row, col };
            let start = addr_map.tile_start_frame(&pos, TileType::Logic);
            decode_one_tile(cram, &mut tiles, row, col + 1, "logic", start, 7, 188);
        }
    }

    for row in 0..num_rows {
        let pos = TilePos { row, col: num_cols + 1 };
        let start = addr_map.tile_start_frame(&pos, TileType::Io);
        decode_one_tile(cram, &mut tiles, row, num_cols + 1, "io", start, 3, 440);
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
    bits_per_sub: u32,
) {
    let mut non_zero_words = Vec::new();
    let mut wiring = Vec::new();
    for fi in 0..num_frames {
        let frame = cram.get_frame(start_frame + fi);
        for wi in 0..33u32 {
            let word = frame.get_word(wi as usize);
            if word == 0 {
                continue;
            }
            non_zero_words.push((fi, wi, word));
            // Decode wiring from set bits within valid range
            let remaining_base = wi * BITS_PER_WORD as u32;
            if remaining_base < bits_per_sub {
                for bit in 0..BITS_PER_WORD as u32 {
                    if (word >> bit) & 1 == 1 {
                        let bit_index = fi * bits_per_sub + remaining_base + bit;
                        wiring.push(DecodedWiring { bit_index });
                    }
                }
            }
        }
    }
    tiles.push(DecodedTile {
        row,
        col,
        tile_type,
        non_zero_words,
        wiring,
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
            assert!(t.wiring.is_empty(), "empty CRAM should have no wiring");
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

    #[test]
    fn test_decode_synckey_zero() {
        let dev = Ice40Device::HX1K;
        let cram = Cram::new(dev);
        let key = decode_synckey(&cram);
        assert_eq!(key, 0, "empty CRAM synckey should be 0");
    }

    #[test]
    fn test_decode_synckey_known() {
        use v2f_bitstream::cram::Cram;
        let dev = Ice40Device::HX1K;
        let mut cram = Cram::new(dev);
        let last_frame = cram.get_frame_mut(cram.num_frames() - 1);
        let fb = FRAME_BITS;
        // Set synckey = 0x12345678
        let key_bytes = 0x12345678u32.to_le_bytes();
        for (i, &b) in key_bytes.iter().enumerate() {
            for bit in 0..8 {
                if (b >> bit) & 1 == 1 {
                    let pos = (fb - 1) - (i * 8 + bit);
                    last_frame.set_bit(pos);
                }
            }
        }
        assert_eq!(decode_synckey(&cram), 0x12345678);
    }

    #[test]
    fn test_wiring_decode_logic_tile() {
        use v2f_bitstream::cram::Cram;
        let dev = Ice40Device::HX1K;
        let mut cram = Cram::new(dev);
        // Simulate apply_asc_to_cram: bit_index=0, value=128
        // frame_sub = 0 / 188 % 7 = 0
        // remaining = 0 % 188 = 0
        // word = 0 / 40 = 0
        // bit = 0 % 40 = 0
        // addr = resolve(TilePos{1,1}, Logic, 0, 0, 0) = frame 125, word 0, bit 0
        let addr = CramAddrMap::new(dev).resolve(
            &TilePos { row: 1, col: 1 },
            TileType::Logic,
            0, 0, 0,
        );
        let f = cram.get_frame_mut(addr.frame);
        f.set_bit((addr.word * 40 + addr.bit) as usize);

        let tiles = decode_cram(&cram, dev);
        // Find logic tile at row=1, col=2 (col+1)
        let lt = tiles.iter().find(|t| t.row == 1 && t.col == 2 && t.tile_type == "logic").unwrap();
        assert_eq!(lt.wiring.len(), 1);
        assert_eq!(lt.wiring[0].bit_index, 0);
    }

    #[test]
    fn test_wiring_decode_io_tile() {
        use v2f_bitstream::cram::Cram;
        let dev = Ice40Device::HX1K;
        let mut cram = Cram::new(dev);
        let addr = CramAddrMap::new(dev).resolve(
            &TilePos { row: 5, col: 0 },
            TileType::Io,
            0, 0, 0,
        );
        let f = cram.get_frame_mut(addr.frame);
        f.set_bit((addr.word * 40 + addr.bit) as usize);

        let tiles = decode_cram(&cram, dev);
        let it = tiles.iter().find(|t| t.row == 5 && t.col == 0 && t.tile_type == "io").unwrap();
        assert_eq!(it.wiring.len(), 1);
        assert_eq!(it.wiring[0].bit_index, 0);
    }
}
