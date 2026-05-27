/// FTDI FT2232H 抽象驅動
///
/// 支援兩種 backend:
/// - Mock: 純 Rust 模擬 (測試用, 預設)
/// - D2XX: libftd2xx-sys 實際硬體驅動 (feature = "ftdi")

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FtdiInterface {
    A,
    B,
}

#[derive(Debug, Clone)]
pub struct FtdiConfig {
    pub vid: u16,
    pub pid: u16,
    pub interface: FtdiInterface,
    pub baud_rate: u32,
}

impl Default for FtdiConfig {
    fn default() -> Self {
        FtdiConfig {
            vid: 0x0403,  // FTDI
            pid: 0x6010,  // FT2232H
            interface: FtdiInterface::A,
            baud_rate: 30_000_000,
        }
    }
}

pub trait FtdiIo {
    fn open(config: &FtdiConfig) -> Result<Self, String> where Self: Sized;
    fn write(&mut self, data: &[u8]) -> Result<(), String>;
    fn read(&mut self, len: usize) -> Result<Vec<u8>, String>;
    fn close(&mut self) -> Result<(), String>;
}

#[derive(Debug, Clone)]
pub struct MockFtdi {
    pub written: Vec<u8>,
    pub read_data: Vec<u8>,
    pub read_pos: usize,
    pub is_open: bool,
}

impl MockFtdi {
    pub fn new() -> Self {
        MockFtdi { written: Vec::new(), read_data: Vec::new(), read_pos: 0, is_open: false }
    }

    pub fn enqueue_read(&mut self, data: Vec<u8>) {
        self.read_data = data;
        self.read_pos = 0;
    }
}

impl FtdiIo for MockFtdi {
    fn open(_config: &FtdiConfig) -> Result<Self, String> {
        Ok(MockFtdi { written: Vec::new(), read_data: Vec::new(), read_pos: 0, is_open: true })
    }

    fn write(&mut self, data: &[u8]) -> Result<(), String> {
        self.written.extend_from_slice(data);
        Ok(())
    }

    fn read(&mut self, len: usize) -> Result<Vec<u8>, String> {
        let avail = self.read_data.len().saturating_sub(self.read_pos);
        let take = len.min(avail);
        let data = self.read_data[self.read_pos..self.read_pos + take].to_vec();
        self.read_pos += take;
        Ok(data)
    }

    fn close(&mut self) -> Result<(), String> {
        self.is_open = false;
        Ok(())
    }
}

#[cfg(feature = "ftdi")]
pub mod real {
    use super::*;

    pub struct RealFtdi {
        // 使用外部 libftd2xx-sys crate
        // handle: ft d2xx::FtHandle,
    }

    impl FtdiIo for RealFtdi {
        fn open(_config: &FtdiConfig) -> Result<Self, String> {
            Err("libftd2xx-sys 尚未整合".into())
        }

        fn write(&mut self, _data: &[u8]) -> Result<(), String> {
            Err("libftd2xx-sys 尚未整合".into())
        }

        fn read(&mut self, _len: usize) -> Result<Vec<u8>, String> {
            Err("libftd2xx-sys 尚未整合".into())
        }

        fn close(&mut self) -> Result<(), String> {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_open() {
        let dev = MockFtdi::open(&FtdiConfig::default()).unwrap();
        assert!(dev.is_open);
    }

    #[test]
    fn test_mock_write_read() {
        let mut dev = MockFtdi::open(&FtdiConfig::default()).unwrap();
        dev.write(b"hello").unwrap();
        assert_eq!(dev.written, b"hello");
        dev.enqueue_read(vec![0x01, 0x02]);
        let data = dev.read(2).unwrap();
        assert_eq!(data, vec![0x01, 0x02]);
    }

    #[test]
    fn test_mock_close() {
        let mut dev = MockFtdi::open(&FtdiConfig::default()).unwrap();
        dev.close().unwrap();
        assert!(!dev.is_open);
    }

    #[test]
    fn test_default_config() {
        let cfg = FtdiConfig::default();
        assert_eq!(cfg.vid, 0x0403);
        assert_eq!(cfg.pid, 0x6010);
        assert_eq!(cfg.baud_rate, 30_000_000);
    }
}
