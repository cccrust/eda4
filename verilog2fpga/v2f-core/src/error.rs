use std::fmt;

#[derive(Debug)]
pub enum V2fError {
    ToolNotFound(String),
    SynthesisFailed(String),
    PnrFailed(String),
    PackFailed(String),
    ProgFailed(String),
    Io(std::io::Error),
    Config(String),
}

impl fmt::Display for V2fError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            V2fError::ToolNotFound(tool) => {
                write!(f, "工具未找到: {tool}。請先安裝: brew install {tool}")
            }
            V2fError::SynthesisFailed(msg) => write!(f, "綜合失敗: {msg}"),
            V2fError::PnrFailed(msg) => write!(f, "佈局佈線失敗: {msg}"),
            V2fError::PackFailed(msg) => write!(f, "打包失敗: {msg}"),
            V2fError::ProgFailed(msg) => write!(f, "燒錄失敗: {msg}"),
            V2fError::Io(e) => write!(f, "IO 錯誤: {e}"),
            V2fError::Config(msg) => write!(f, "設定錯誤: {msg}"),
        }
    }
}

impl std::error::Error for V2fError {}

impl From<std::io::Error> for V2fError {
    fn from(e: std::io::Error) -> Self {
        V2fError::Io(e)
    }
}

pub type V2fResult<T> = Result<T, V2fError>;
