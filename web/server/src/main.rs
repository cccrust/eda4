use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_ws::Message;
use futures_util::StreamExt;
use serde::Deserialize;
use tokio::sync::mpsc;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use eda4_web_server::{handle_request, protocol::{Request, Response}, SpiceAnalysisType};

#[derive(Debug, Deserialize)]
struct ApiVerilogSim {
    code: String,
    top: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiVerilogPnr {
    code: String,
    device: Option<String>,
    top: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiSpiceAnalyze {
    code: String,
    analysis: Option<String>,
    ac_freq_start: Option<f64>,
    ac_freq_end: Option<f64>,
    ac_points: Option<usize>,
    tran_start: Option<f64>,
    tran_end: Option<f64>,
    tran_step: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct ApiBitstreamDecode {
    bin_base64: String,
}

#[derive(Clone)]
struct AppState {}

async fn ws_handler(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let (response, mut session, msg_stream) = actix_ws::handle(&req, stream)?;

    let (tx, mut rx) = mpsc::channel::<String>(32);
    let session_send = session.clone();

    let _tx = tx.clone();
    actix_rt::spawn(async move {
        let mut s = session_send;
        while let Some(msg) = rx.recv().await {
            match s.text(msg).await {
                Ok(()) => {}
                Err(e) => { error!("WS send error: {}", e); break; }
            }
        }
    });

    let mut msg_stream = msg_stream;
    loop {
        match msg_stream.next().await {
            Some(Ok(msg)) => {
                match msg {
                    Message::Text(text) => {
                        let req: Result<Request, _> = serde_json::from_str(&text);
                        let resp = match req {
                            Ok(r) => handle_request(r),
                            Err(e) => Response::Error { error: format!("JSON parse error: {}", e) },
                        };
                        let resp_json = serde_json::to_string(&resp)
                            .unwrap_or_else(|_| r#"{"type":"error","error":"serialization failed"}"#.to_string());
                        if tx.send(resp_json).await.is_err() {
                            break;
                        }
                    }
                    Message::Ping(bytes) => {
                        match session.pong(&bytes).await {
                            Ok(()) => {}
                            Err(e) => { error!("WS pong error: {}", e); }
                        }
                    }
                    Message::Close(reason) => {
                        let _ = session.close(reason).await;
                        break;
                    }
                    _ => {}
                }
            }
            Some(Err(e)) => {
                error!("WebSocket error: {}", e);
                break;
            }
            None => break,
        }
    }

    Ok(response)
}

async fn api_verilog_sim(_state: web::Data<AppState>, body: web::Json<ApiVerilogSim>) -> impl Responder {
    let resp = handle_request(Request::VerilogSim {
        code: body.code.clone(),
        top: body.top.clone(),
    });
    HttpResponse::Ok().json(&resp)
}

async fn api_verilog_pnr(_state: web::Data<AppState>, body: web::Json<ApiVerilogPnr>) -> impl Responder {
    let resp = handle_request(Request::VerilogPnr {
        code: body.code.clone(),
        device: body.device.clone(),
        top: body.top.clone(),
    });
    HttpResponse::Ok().json(&resp)
}

async fn api_spice_analyze(_state: web::Data<AppState>, body: web::Json<ApiSpiceAnalyze>) -> impl Responder {
    let analysis = match body.analysis.as_deref() {
        Some("dc") => SpiceAnalysisType::Dc,
        Some("ac") => SpiceAnalysisType::Ac,
        Some("transient") => SpiceAnalysisType::Transient,
        _ => SpiceAnalysisType::All,
    };
    let resp = handle_request(Request::SpiceAnalyze {
        code: body.code.clone(),
        analysis,
        ac_freq_start: body.ac_freq_start,
        ac_freq_end: body.ac_freq_end,
        ac_points: body.ac_points,
        tran_start: body.tran_start,
        tran_end: body.tran_end,
        tran_step: body.tran_step,
    });
    HttpResponse::Ok().json(&resp)
}

async fn api_bitstream_decode(_state: web::Data<AppState>, body: web::Json<ApiBitstreamDecode>) -> impl Responder {
    let resp = handle_request(Request::BitstreamDecode {
        bin_base64: body.bin_base64.clone(),
    });
    HttpResponse::Ok().json(&resp)
}

async fn index_handler() -> impl Responder {
    let html = include_str!("../../frontend/index.html");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("EDA4 Web Server starting on http://0.0.0.0:8080");
    info!("REST: POST /api/verilog/simulate, /api/verilog/pnr, /api/spice/analyze, /api/bitstream/decode");
    info!("WebSocket: GET /ws");

    let bind_addr = "0.0.0.0:8080";
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(AppState {}))
            .route("/", web::get().to(index_handler))
            .route("/ws", web::get().to(ws_handler))
            .route("/api/verilog/simulate", web::post().to(api_verilog_sim))
            .route("/api/verilog/pnr", web::post().to(api_verilog_pnr))
            .route("/api/spice/analyze", web::post().to(api_spice_analyze))
            .route("/api/bitstream/decode", web::post().to(api_bitstream_decode))
            .service(actix_files::Files::new("/js", "frontend/js").show_files_listing())
    })
    .bind(bind_addr)?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sr(code: String, analysis: SpiceAnalysisType) -> Request {
        Request::SpiceAnalyze {
            code,
            analysis,
            ac_freq_start: None,
            ac_freq_end: None,
            ac_points: None,
            tran_start: None,
            tran_end: None,
            tran_step: None,
        }
    }

    fn vr(code: String, device: Option<String>) -> Request {
        Request::VerilogPnr { code, device, top: None }
    }

    #[tokio::test]
    async fn test_spice_resistor_divider() {
        let resp = handle_request(sr("V1 Vin gnd DC 10\nR1 Vin Vout 1k\nR2 Vout gnd 1k\n.END".into(), SpiceAnalysisType::Dc));
        match resp {
            Response::SpiceResult { dc: Some(dc), .. } => {
                assert!((dc.node_voltages[2] - 5.0).abs() < 0.001);
            }
            _ => panic!("Expected SpiceResult with DC"),
        }
    }

    #[tokio::test]
    async fn test_spice_rc_transient() {
        let resp = handle_request(sr("V1 Vin gnd DC 5\nR1 Vin Vout 1k\nC1 Vout gnd 1u\n.END".into(), SpiceAnalysisType::Transient));
        match resp {
            Response::SpiceResult { transient: Some(t), .. } => {
                assert!(!t.time_points.is_empty());
                assert!(!t.node_voltages.is_empty());
            }
            _ => panic!("Expected SpiceResult with transient"),
        }
    }

    #[tokio::test]
    async fn test_spice_ac() {
        let resp = handle_request(sr("V1 Vin gnd DC 5\nR1 Vin Vout 1k\nC1 Vout gnd 1u\n.END".into(), SpiceAnalysisType::Ac));
        match resp {
            Response::SpiceResult { ac: Some(ac), .. } => {
                assert!(!ac.frequencies.is_empty());
            }
            _ => panic!("Expected SpiceResult with AC"),
        }
    }

    #[tokio::test]
    async fn test_spice_all() {
        let resp = handle_request(sr("V1 Vin gnd DC 5\nR1 Vin Vout 1k\n.END".into(), SpiceAnalysisType::All));
        match resp {
            Response::SpiceResult { dc: Some(_), ac: Some(_), transient: Some(_), ascii_circuit, .. } => {
                assert!(!ascii_circuit.is_empty());
            }
            _ => panic!("Expected full SpiceResult"),
        }
    }

    #[tokio::test]
    async fn test_spice_parse_error() {
        let resp = handle_request(sr("INVALID".into(), SpiceAnalysisType::Dc));
        match resp {
            Response::Error { .. } => {}
            Response::SpiceResult { dc: Some(dc), .. } => {
                assert!(dc.node_voltages.len() <= 1, "Invalid SPICE should produce at most 1 node");
            }
            _ => panic!("Expected Error or SpiceResult for invalid SPICE"),
        }
    }

    #[tokio::test]
    async fn test_spice_ohm() {
        let resp = handle_request(sr("V1 Vin gnd DC 10\nR1 Vin gnd 500\n.END".into(), SpiceAnalysisType::Dc));
        match resp {
            Response::SpiceResult { dc: Some(dc), .. } => {
                assert!((dc.node_voltages[1] - 10.0).abs() < 0.001);
            }
            _ => panic!("Expected SpiceResult with DC"),
        }
    }

    #[tokio::test]
    async fn test_verilog_pnr_simple() {
        let resp = handle_request(vr("module top(input a, output y); assign y = a; endmodule".into(), Some("hx1k".into())));
        match resp {
            Response::VerilogPnrResult { json, asc, bin_base64, device } => {
                assert!(!json.is_empty());
                assert!(!asc.is_empty());
                assert!(!bin_base64.is_empty());
                assert_eq!(device, "hx1k");
            }
            _ => panic!("Expected VerilogPnrResult"),
        }
    }

    #[tokio::test]
    async fn test_verilog_pnr_blinky() {
        let resp = handle_request(vr("module blinky(input clk, output reg led);\nreg [25:0] counter;\nalways @(posedge clk) counter <= counter + 1;\nassign led = counter[25];\nendmodule".into(), Some("hx8k".into())));
        match resp {
            Response::VerilogPnrResult { bin_base64, .. } => assert!(!bin_base64.is_empty()),
            _ => panic!("Expected VerilogPnrResult"),
        }
    }

    #[tokio::test]
    async fn test_verilog_pnr_invalid_device() {
        let resp = handle_request(vr("module top(input a, output y); assign y = a; endmodule".into(), Some("bad".into())));
        assert!(matches!(resp, Response::Error { .. }));
    }

    #[tokio::test]
    async fn test_verilog_pnr_default_device() {
        let resp = handle_request(vr("module top(input a, output y); assign y = a; endmodule".into(), None));
        match resp {
            Response::VerilogPnrResult { device, .. } => assert_eq!(device, "hx8k"),
            _ => panic!("Expected VerilogPnrResult"),
        }
    }

    #[tokio::test]
    async fn test_verilog_pnr_all_devices() {
        for d in &["hx8k", "lp1k", "up5k", "hx4k", "hx1k"] {
            let resp = handle_request(vr("module top(input a, output y); assign y = a; endmodule".into(), Some(d.to_string())));
            match resp {
                Response::VerilogPnrResult { device, .. } => assert_eq!(device.as_str(), *d),
                _ => panic!("Expected VerilogPnrResult for {}", d),
            }
        }
    }

    #[tokio::test]
    async fn test_verilog_sim_generates_rust() {
        let resp = handle_request(Request::VerilogSim {
            code: "module foo(a, b, s);\n  input a;\n  input b;\n  output s;\n  and g(s, a, b);\nendmodule".into(),
            top: None,
        });
        match resp {
            Response::VerilogSimResult { generated_rust, .. } => {
                assert!(!generated_rust.is_empty(), "Should generate some Rust");
            }
            _ => panic!("Expected VerilogSimResult"),
        }
    }

    #[tokio::test]
    async fn test_bitstream_decode_invalid_base64() {
        let resp = handle_request(Request::BitstreamDecode { bin_base64: "!!!invalid!!!".into() });
        assert!(matches!(resp, Response::Error { .. }));
    }

    #[tokio::test]
    async fn test_bitstream_decode_empty_hx1k() {
        use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
        use v2f_bitstream::{pack_bitstream, Cram};
        use v2f_db::ice40::Ice40Device;
        let dev = Ice40Device::HX1K;
        let cram = Cram::new(dev);
        let bin = pack_bitstream(&cram);
        let b64 = BASE64.encode(&bin);
        let resp = handle_request(Request::BitstreamDecode { bin_base64: b64 });
        match resp {
            Response::BitstreamDecodeResult { device, crc_valid, tiles } => {
                assert_eq!(device, "hx1k");
                assert!(crc_valid);
                assert!(tiles.get("format").is_some());
            }
            _ => panic!("Expected BitstreamDecodeResult"),
        }
    }
}