/// iCE40 CRAM/SRAM 直接載入協定
///
/// 遵循 iCE40 程式化流程：
///   1. Reset CRAM
///   2. Send 8 dummy bytes (0x00)
///   3. Send bitstream payload
///   4. Send CRC (4 bytes)
///   5. Send 48 dummy clocks
///   6. Wakeup

use crate::spi::SpiFlash;

pub fn crc32(data: &[u8]) -> [u8; 4] {
    let mut crc = 0xFFFFFFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 { crc = (crc >> 1) ^ 0xEDB88320; }
            else { crc >>= 1; }
        }
    }
    (crc ^ 0xFFFFFFFF).to_le_bytes()
}

pub struct Ice40Config;

impl Ice40Config {
    // SRAM 直接載入 (透過類 SPI bitbanging)
    pub fn load_sram(write_byte: &mut dyn FnMut(u8), bitstream: &[u8]) -> Result<(), String> {
        // 1. Reset CRAM: 至少 8 個 1 的 clock
        write_byte(0xFF);
        write_byte(0xFF);
        write_byte(0xFF);
        write_byte(0xFF);

        // 2. 8 dummy bytes
        for _ in 0..8 { write_byte(0x00); }

        // 3. Bitstream payload
        for &b in bitstream { write_byte(b); }

        // 4. CRC
        let crc = crc32(bitstream);
        for &b in &crc { write_byte(b); }

        // 5. 48 dummy clocks (6 bytes)
        for _ in 0..6 { write_byte(0x00); }

        // 6. Wakeup (送出 0x00 的最後一些 clock)
        write_byte(0x00);

        Ok(())
    }

    // 從 SPI flash 載入配置
    pub fn load_from_flash(flash: &SpiFlash, addr: usize, len: usize) -> Result<Vec<u8>, String> {
        Ok(flash.read(addr, len))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32_known() {
        let data = b"123456789";
        let crc = crc32(data);
        assert_eq!(u32::from_le_bytes(crc), 0xCBF43926);
    }

    #[test]
    fn test_crc32_empty() {
        let crc = crc32(b"");
        assert_eq!(u32::from_le_bytes(crc), 0x00000000);
    }

    #[test]
    fn test_load_sram_bytes_written() {
        let mut written = Vec::new();
        {
            let mut writer = |b: u8| written.push(b);
            let bs = vec![0xAA; 16];
            Ice40Config::load_sram(&mut writer, &bs).unwrap();
        }
        assert!(!written.is_empty());
        // 第一個 4 位元組是 reset (0xFF)
        assert_eq!(written[0], 0xFF);
        // 接下來 8 位元組是 dummy (0x00)
        assert_eq!(written[4], 0x00);
        // 然後是 bitstream payload
        assert_eq!(written[12], 0xAA);
    }
}
