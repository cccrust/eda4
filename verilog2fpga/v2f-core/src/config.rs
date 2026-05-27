use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub device: Option<String>,
    pub top: Option<String>,
    pub pcf: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            device: Some("hx8k".into()),
            top: None,
            pcf: None,
        }
    }
}
