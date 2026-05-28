use v2f_bitstream::cram::Cram;
use v2f_bitstream::frame::{Frame, FRAME_BITS, FRAME_BYTES};
use v2f_bitstream::pack::crc32;
use v2f_db::ice40::Ice40Device;

/// 解析後的 BIN 檔案
#[derive(Debug)]
pub struct BinFile {
    pub device: Ice40Device,
    pub cram: Cram,
    pub crc_valid: bool,
}

/// 將 BIN bitstream 解析為 CRAM
///
/// 格式: [32 zero preamble] [bit_count:u32 LE] [frame_data] [CRC32:u32 LE]
pub fn parse_bin(data: &[u8]) -> Result<BinFile, String> {
    if data.len() < 40 {
        return Err(format!("file too short: {} bytes (min 40)", data.len()));
    }
    if &data[..32] != &[0u8; 32] {
        return Err("invalid preamble: first 32 bytes must be zero".into());
    }
    let total_bits = u32::from_le_bytes(data[32..36].try_into().unwrap()) as usize;
    if total_bits % FRAME_BITS != 0 {
        return Err(format!(
            "bit count {} not multiple of frame size {}",
            total_bits, FRAME_BITS
        ));
    }
    let total_frames = total_bits / FRAME_BITS;
    let device = detect_device(total_frames as u32)?;
    let payload_len = total_bits / 8;
    let expected_total = 36 + payload_len + 4;
    if data.len() < expected_total {
        return Err(format!(
            "file truncated: {} bytes, expected at least {}",
            data.len(),
            expected_total
        ));
    }
    if data.len() > expected_total {
        return Err(format!(
            "file too large: {} bytes, expected {}",
            data.len(),
            expected_total
        ));
    }
    let payload = &data[36..36 + payload_len];
    let stored_crc = u32::from_le_bytes(
        data[36 + payload_len..36 + payload_len + 4]
            .try_into()
            .unwrap(),
    );
    let computed_crc = crc32(payload);
    let crc_valid = stored_crc == computed_crc;
    if !crc_valid {
        return Err(format!(
            "CRC mismatch: stored {:#010x}, computed {:#010x}",
            stored_crc, computed_crc
        ));
    }
    let mut cram = Cram::new(device);
    for i in 0..total_frames {
        let offset = i * FRAME_BYTES;
        let frame = Frame::from_bytes(&payload[offset..offset + FRAME_BYTES]);
        *cram.get_frame_mut(i as u32) = frame;
    }
    Ok(BinFile {
        device,
        cram,
        crc_valid,
    })
}

fn detect_device(total_frames: u32) -> Result<Ice40Device, String> {
    for dev in &[
        Ice40Device::HX1K,
        Ice40Device::HX4K,
        Ice40Device::HX8K,
        Ice40Device::LP1K,
        Ice40Device::UP5K,
    ] {
        if dev.total_frames() == total_frames {
            return Ok(*dev);
        }
    }
    Err(format!(
        "unknown device: {} frames (expected: HX1K={}, HX4K={}, HX8K={}, LP1K={}, UP5K={})",
        total_frames,
        Ice40Device::HX1K.total_frames(),
        Ice40Device::HX4K.total_frames(),
        Ice40Device::HX8K.total_frames(),
        Ice40Device::LP1K.total_frames(),
        Ice40Device::UP5K.total_frames(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_hx1k() {
        assert_eq!(
            detect_device(Ice40Device::HX1K.total_frames()).unwrap(),
            Ice40Device::HX1K
        );
    }

    #[test]
    fn test_detect_hx8k() {
        assert_eq!(
            detect_device(Ice40Device::HX8K.total_frames()).unwrap(),
            Ice40Device::HX8K
        );
    }

    #[test]
    fn test_detect_unknown() {
        assert!(detect_device(9999).is_err());
    }
}
