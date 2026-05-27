/// iCE40 程式化流程 (SPI Slave / JTAG 模式)

use crate::jtag::{JtagStateMachine, TapState};
use crate::spi::SpiFlash;

#[derive(Debug, Clone, Copy)]
pub enum Ice40JtagInstruction {
    Extest  = 0x00,
    Sample  = 0x01,
    UsrCode = 0x02,
    Usr0    = 0x03,
    Usr1    = 0x0B,
    Highz   = 0x07,
    Clamp   = 0x0A,
    Intest  = 0x0C,
    Bypass  = 0x3F,
}

impl Ice40JtagInstruction {
    pub fn code(&self) -> u8 { *self as u8 }
    pub fn len(&self) -> u8 {
        match self {
            Ice40JtagInstruction::Bypass => 1,
            _ => 5,
        }
    }
}

pub struct Ice40Programmer;

impl Ice40Programmer {
    pub fn program_cram_jtag(jtag: &mut JtagStateMachine, bitstream: &[u8]) -> Result<(), String> {
        jtag.go_to(TapState::RunTestIdle);
        jtag.shift_ir(Ice40JtagInstruction::Usr1.code(), Ice40JtagInstruction::Usr1.len());
        let mut framed = Vec::new();
        framed.extend_from_slice(&[0x00u8; 8]);
        framed.extend_from_slice(bitstream);
        jtag.shift_dr(&framed, true);
        jtag.go_to(TapState::RunTestIdle);
        Ok(())
    }

    pub fn program_spi_flash(bitstream: &[u8], flash: &mut SpiFlash) -> Result<(), String> {
        let page_size = flash.page_size;
        for (offset, chunk) in bitstream.chunks(page_size).enumerate() {
            flash.page_program(offset * page_size, chunk);
        }
        Ok(())
    }

    pub fn verify_flash(bitstream: &[u8], flash: &SpiFlash) -> bool {
        for (offset, chunk) in bitstream.chunks(256).enumerate() {
            let readback = flash.read(offset * 256, chunk.len());
            if readback != chunk { return false; }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spi::SpiFlash;

    #[test]
    fn test_program_cram_jtag() {
        let mut jtag = JtagStateMachine::new();
        let bs = vec![0xAA; 1024];
        assert!(Ice40Programmer::program_cram_jtag(&mut jtag, &bs).is_ok());
    }

    #[test]
    fn test_program_spi_flash() {
        let mut flash = SpiFlash::new(4096);
        let bs = vec![0x55; 512];
        Ice40Programmer::program_spi_flash(&bs, &mut flash).unwrap();
        assert!(Ice40Programmer::verify_flash(&bs, &flash));
    }

    #[test]
    fn test_verify_fails_on_mismatch() {
        let mut flash = SpiFlash::new(4096);
        let bs = vec![0x55; 256];
        Ice40Programmer::program_spi_flash(&bs, &mut flash).unwrap();
        assert!(!Ice40Programmer::verify_flash(&[0xAA; 256], &flash));
    }
}
