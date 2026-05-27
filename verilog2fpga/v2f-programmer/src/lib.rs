pub mod jtag;
pub mod spi;
pub mod ftdi;
pub mod ice40_prog;
pub mod ice40_cfg;

pub use ftdi::{FtdiConfig, FtdiInterface, FtdiIo, MockFtdi};
pub use ice40_prog::Ice40Programmer;
pub use ice40_cfg::Ice40Config;

pub fn check_driver_support() -> Vec<&'static str> {
    let drivers = vec!["mock (模擬)"];
    #[cfg(feature = "ftdi")]
    let drivers = { let mut d = drivers; d.push("ftdi (FT2232H)"); d };
    drivers
}
