use crate::protocol::{Request, Response};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

pub fn handle(req: Request) -> Response {
    match req {
        Request::BitstreamDecode { bin_base64 } => handle_decode(bin_base64),
        _ => Response::Error { error: "Unknown request type for bitstream_decode service".into() },
    }
}

fn handle_decode(bin_base64: String) -> Response {
    let bin_data = match BASE64.decode(&bin_base64) {
        Ok(d) => d,
        Err(e) => return Response::Error { error: format!("Base64 decode error: {}", e) },
    };

    let bin_file = match v2f_bitdecode::parse_bin(&bin_data) {
        Ok(b) => b,
        Err(e) => return Response::Error { error: format!("Bitstream parse error: {}", e) },
    };

    let decoded_tiles = v2f_bitdecode::decode_cram(&bin_file.cram, bin_file.device);
    let synckey = v2f_bitdecode::decode_synckey(&bin_file.cram);
    let tiles_json = v2f_bitdecode::tiles_to_json(
        &decoded_tiles,
        bin_file.device,
        "uploaded",
        bin_file.crc_valid,
        Some(synckey),
        "wiring",
    );

    Response::BitstreamDecodeResult {
        device: bin_file.device.name().to_string(),
        crc_valid: bin_file.crc_valid,
        tiles: tiles_json,
    }
}