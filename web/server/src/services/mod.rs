pub mod spice;
pub mod verilog_pnr;
pub mod verilog_sim;
pub mod bitstream_decode;

pub use crate::protocol::{Request, Response, SpiceAnalysisType};
pub use crate::protocol::{DcResult, AcResult, TransientResult};

pub fn handle_request(req: Request) -> Response {
    match &req {
        Request::VerilogSim { .. } => verilog_sim::handle(req),
        Request::VerilogPnr { .. } => verilog_pnr::handle(req),
        Request::SpiceAnalyze { .. } => spice::handle(req),
        Request::BitstreamDecode { .. } => bitstream_decode::handle(req),
    }
}