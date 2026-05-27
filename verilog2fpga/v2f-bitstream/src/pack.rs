/// iCE40 位元流打包器
///
/// 產生標準 iCE40 bitstream 格式：
///
/// ```text
/// [0x00 × 32]         前導空白
/// [BitCount: u32 LE]  總位元數
/// [Frame0..FrameN]    每個 Frame = 165 bytes
/// [CRC32: u32 LE]     校驗碼
/// ```
///
/// 參考：Project IceStorm (icestorm) icepack 實作。

use crate::cram::Cram;
use crate::frame::FRAME_BITS;

/// 前導空白長度 (bytes)
pub const PREAMBLE_SIZE: usize = 32;

/// 計算標準 CRC-32 (IEEE 802.3 / PKZIP)
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFFFFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

/// 將 CRAM 打包為完整 bitstream
pub fn pack_bitstream(cram: &Cram) -> Vec<u8> {
    let mut bitstream = Vec::new();

    // 1. 前導空白
    bitstream.extend(std::iter::repeat(0u8).take(PREAMBLE_SIZE));

    // 2. 位元數 (CRAM 資料的總位元數)
    let total_bits = cram.num_frames() * FRAME_BITS as u32;
    bitstream.extend_from_slice(&total_bits.to_le_bytes());

    // 3. CRAM 資料 (所有 Frame)
    let cram_data = cram.to_bytes();
    bitstream.extend_from_slice(&cram_data);

    // 4. CRC32 (僅計算 CRAM 資料部分)
    let crc = crc32(&cram_data);
    bitstream.extend_from_slice(&crc.to_le_bytes());

    bitstream
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32_known() {
        // CRC-32 of "123456789" should be 0xCBF43926
        let data = b"123456789";
        assert_eq!(crc32(data), 0xCBF43926);
    }

    #[test]
    fn test_crc32_empty() {
        assert_eq!(crc32(b""), 0x00000000);
    }

    #[test]
    fn test_pack_minimal() {
        use v2f_db::ice40::Ice40Device;
        let dev = Ice40Device::HX1K;
        let cram = Cram::new(dev);
        let bs = pack_bitstream(&cram);

        // preamble
        assert_eq!(&bs[..PREAMBLE_SIZE], &[0u8; PREAMBLE_SIZE]);

        // bit count
        let bit_count_pos = PREAMBLE_SIZE;
        let total_bits =
            u32::from_le_bytes(bs[bit_count_pos..bit_count_pos + 4].try_into().unwrap());
        assert_eq!(total_bits, dev.total_frames() * FRAME_BITS as u32);

        // CRC at end
        let crc_pos = bs.len() - 4;
        let stored_crc =
            u32::from_le_bytes(bs[crc_pos..crc_pos + 4].try_into().unwrap());
        
        let cram_data = &bs[bit_count_pos + 4..crc_pos];
        let expected_crc = crc32(cram_data);
        assert_eq!(stored_crc, expected_crc);
    }
}
