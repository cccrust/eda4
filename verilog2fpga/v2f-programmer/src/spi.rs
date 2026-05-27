/// SPI flash 指令集 (標準 + iCE40 專用)
///
/// 用於 iCE40 SPI Slave 模式燒錄。

#[derive(Debug, Clone, Copy)]
pub enum SpiCommand {
    WriteEnable      = 0x06,
    WriteDisable     = 0x04,
    ReadStatus       = 0x05,
    WriteStatus      = 0x01,
    ReadData         = 0x03,
    FastRead         = 0x0B,
    PageProgram      = 0x02,
    SectorErase      = 0x20,
    BlockErase32K    = 0x52,
    BlockErase64K    = 0xD8,
    ChipErase        = 0xC7,
    DeepPowerDown    = 0xB9,
    ReleasePowerDown = 0xAB,
    Enter4ByteAddr   = 0xB7,
    Exit4ByteAddr    = 0xE9,
}

impl SpiCommand {
    pub fn code(&self) -> u8 { *self as u8 }
}

#[derive(Debug)]
pub struct SpiFlash {
    pub page_size: usize,
    pub sector_size: usize,
    pub num_sectors: usize,
    pub memory: Vec<u8>,
}

impl SpiFlash {
    pub fn new(size_bytes: usize) -> Self {
        SpiFlash {
            page_size: 256,
            sector_size: 4096,
            num_sectors: size_bytes / 4096,
            memory: vec![0xFF; size_bytes],
        }
    }

    pub fn erase_chip(&mut self) {
        self.memory.fill(0xFF);
    }

    pub fn erase_sector(&mut self, addr: usize) {
        let start = (addr / self.sector_size) * self.sector_size;
        let end = (start + self.sector_size).min(self.memory.len());
        self.memory[start..end].fill(0xFF);
    }

    pub fn page_program(&mut self, addr: usize, data: &[u8]) {
        let end = (addr + data.len()).min(self.memory.len());
        if addr < end {
            self.memory[addr..end].copy_from_slice(&data[..end - addr]);
        }
    }

    pub fn read(&self, addr: usize, len: usize) -> Vec<u8> {
        let end = (addr + len).min(self.memory.len());
        self.memory[addr..end].to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spi_flash_init() {
        let flash = SpiFlash::new(1024 * 1024);
        assert_eq!(flash.memory.len(), 1_048_576);
        assert!(flash.memory.iter().all(|&b| b == 0xFF));
    }

    #[test]
    fn test_erase_chip() {
        let mut flash = SpiFlash::new(4096);
        flash.page_program(0, &[0xAA; 256]);
        flash.erase_chip();
        assert!(flash.memory.iter().all(|&b| b == 0xFF));
    }

    #[test]
    fn test_page_program_and_read() {
        let mut flash = SpiFlash::new(4096);
        let data = vec![0x12, 0x34, 0x56, 0x78];
        flash.page_program(100, &data);
        let r = flash.read(100, 4);
        assert_eq!(r, data);
    }

    #[test]
    fn test_spi_command_codes() {
        assert_eq!(SpiCommand::WriteEnable.code(), 0x06);
        assert_eq!(SpiCommand::ChipErase.code(), 0xC7);
        assert_eq!(SpiCommand::PageProgram.code(), 0x02);
    }
}
