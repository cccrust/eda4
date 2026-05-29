use std::panic;
use crate::protocol::{Request, Response};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use v2f_core::Device as CoreDevice;
use v2f_db::ice40::Ice40Device;

pub fn handle(req: Request) -> Response {
    match req {
        Request::VerilogPnr { code, device, top } => handle_pnr(code, device, top),
        _ => Response::Error { error: "Unknown request type for verilog_pnr service".into() },
    }
}

fn device_to_ice40(d: CoreDevice) -> Ice40Device {
    match d {
        CoreDevice::HX1K => Ice40Device::HX1K,
        CoreDevice::HX4K => Ice40Device::HX4K,
        CoreDevice::HX8K => Ice40Device::HX8K,
        CoreDevice::LP1K => Ice40Device::LP1K,
        CoreDevice::UP5K => Ice40Device::UP5K,
    }
}

fn handle_pnr(code: String, device: Option<String>, top: Option<String>) -> Response {
    let dev_str = device.unwrap_or_else(|| "hx8k".to_string());
    let dev: CoreDevice = match dev_str.parse() {
        Ok(d) => d,
        Err(e) => return Response::Error { error: format!("Invalid device '{}': {}", dev_str, e) },
    };
    let top_name = top.unwrap_or_else(|| "top".to_string());
    let ice_dev = device_to_ice40(dev);

    let json = match panic::catch_unwind(|| v2f_synth::synthesize(&code, &top_name)) {
        Ok(j) => j,
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<&str>() { s.to_string() }
                      else if let Some(s) = e.downcast_ref::<String>() { s.clone() }
                      else { "synthesis panic (unsupported Verilog construct)".into() };
            return Response::Error { error: format!("Synthesis error: {}", msg) };
        }
    };

    let asc = match panic::catch_unwind(|| v2f_pnr::run_pnr(&json, dev)) {
        Ok(a) => a,
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<&str>() { s.to_string() }
                      else if let Some(s) = e.downcast_ref::<String>() { s.clone() }
                      else { "PnR panic".into() };
            return Response::Error { error: format!("PnR error: {}", msg) };
        }
    };

    let asc_parsed = match v2f_bitstream::parse_asc(&asc) {
        Ok(a) => a,
        Err(e) => return Response::Error { error: format!("ASC parse error: {}", e) },
    };

    let mut cram = v2f_bitstream::Cram::new(ice_dev);
    v2f_bitstream::apply_asc_to_cram(&asc_parsed, &mut cram, ice_dev);

    let bin = v2f_bitstream::pack_bitstream(&cram);
    let bin_base64 = BASE64.encode(&bin);

    Response::VerilogPnrResult {
        json,
        asc,
        bin_base64,
        device: dev_str,
    }
}