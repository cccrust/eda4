pub mod services;
pub mod protocol;

pub use services::handle_request;
pub use protocol::{Request, Response, SpiceAnalysisType};
pub use protocol::{DcResult, AcResult, TransientResult};