use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Device {
    HX1K,
    HX4K,
    HX8K,
    LP1K,
    UP5K,
}

impl Device {
    pub fn all() -> &'static [Device] {
        &[Device::HX1K, Device::HX4K, Device::HX8K, Device::LP1K, Device::UP5K]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Device::HX1K => "hx1k",
            Device::HX4K => "hx4k",
            Device::HX8K => "hx8k",
            Device::LP1K => "lp1k",
            Device::UP5K => "up5k",
        }
    }

    pub fn nextpnr_flag(&self) -> &'static str {
        match self {
            Device::HX1K => "--hx1k",
            Device::HX4K => "--hx4k",
            Device::HX8K => "--hx8k",
            Device::LP1K => "--lp1k",
            Device::UP5K => "--up5k",
        }
    }
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl FromStr for Device {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hx1k" => Ok(Device::HX1K),
            "hx4k" => Ok(Device::HX4K),
            "hx8k" => Ok(Device::HX8K),
            "lp1k" => Ok(Device::LP1K),
            "up5k" => Ok(Device::UP5K),
            _ => Err(format!(
                "未知裝置: {}。支援: hx1k, hx4k, hx8k, lp1k, up5k",
                s
            )),
        }
    }
}
