use serde_json::{json, Value};
use v2f_db::ice40::Ice40Device;

use crate::cram_decode::DecodedTile;

/// 將 tile 解碼結果轉為 JSON Value
pub fn tiles_to_json(
    tiles: &[DecodedTile],
    device: Ice40Device,
    bin_path: &str,
    crc_valid: bool,
) -> Value {
    let mut tiles_map = serde_json::Map::new();
    let mut summary_logic_used = 0u32;
    let mut summary_io_used = 0u32;

    for t in tiles {
        if !t.non_zero_words.is_empty() {
            match t.tile_type {
                "logic" => summary_logic_used += 1,
                "io" => summary_io_used += 1,
                _ => {}
            }
        }
        let key = format!("{}_{}_{}", t.tile_type, t.col, t.row);
        let mut words = Vec::new();
        for &(fi, wi, val) in &t.non_zero_words {
            words.push(json!({
                "frame_within_tile": fi,
                "word": wi,
                "value_hex": format!("{:#010x}", val),
                "bits_set": count_bits(val),
            }));
        }
        let used = !t.non_zero_words.is_empty();
        tiles_map.insert(key, json!({
            "type": t.tile_type,
            "col": t.col,
            "row": t.row,
            "used": used,
            "non_zero_words": words,
        }));
    }

    json!({
        "format": "v2f-bitdecode-v1",
        "device": device.name(),
        "created_by": "v2f-bitdecode v0.9",
        "bin_source": bin_path,
        "crc_valid": crc_valid,
        "num_tiles": tiles.len(),
        "summary": {
            "logic_tiles_total": tiles.iter().filter(|t| t.tile_type == "logic").count(),
            "logic_tiles_used": summary_logic_used,
            "io_tiles_total": tiles.iter().filter(|t| t.tile_type == "io").count(),
            "io_tiles_used": summary_io_used,
        },
        "tiles": Value::Object(tiles_map),
    })
}

fn count_bits(val: u64) -> u32 {
    val.count_ones()
}

#[cfg(test)]
mod tests {
    use super::*;
    use v2f_bitstream::cram::Cram;
    use crate::cram_decode::decode_cram;

    #[test]
    fn test_empty_cram_json() {
        let dev = Ice40Device::HX1K;
        let cram = Cram::new(dev);
        let tiles = decode_cram(&cram, dev);
        let json = tiles_to_json(&tiles, dev, "test.bin", true);
        assert_eq!(json["device"], "hx1k");
        assert_eq!(json["crc_valid"], true);
        assert_eq!(json["summary"]["logic_tiles_used"], 0);
        assert_eq!(json["summary"]["io_tiles_used"], 0);
    }
}
