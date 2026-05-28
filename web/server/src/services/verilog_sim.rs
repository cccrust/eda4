use crate::protocol::{Request, Response};
use std::panic::catch_unwind;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

const RLIB_CACHE: &str = "/tmp/verilog2rust_rlib";
const BIN_CACHE: &str = "/tmp/verilog2rust_bin";

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn next_temp_id() -> u64 {
    TEMP_COUNTER.fetch_add(1, Ordering::SeqCst)
}

pub fn handle(req: Request) -> Response {
    match req {
        Request::VerilogSim { code, top } => handle_sim(code, top),
        _ => Response::Error { error: "Unknown request type for verilog_sim service".into() },
    }
}

fn build_lib_rlib() -> Result<String, String> {
    let cache = std::path::Path::new(RLIB_CACHE);
    std::fs::create_dir_all(cache).map_err(|e| e.to_string())?;
    let rlib_path = format!("{}/libverilog2rust.rlib", RLIB_CACHE);
    let parser_rlib = format!("{}/libverilog_parser.rlib", RLIB_CACHE);

    if std::path::Path::new(&rlib_path).exists() {
        return Ok(rlib_path);
    }

    let manifest = env!("CARGO_MANIFEST_DIR");

    let parser_src = std::path::Path::new(manifest)
        .join("../../verilog-parser/src/lib.rs");
    let output = Command::new("rustc")
        .args(["--crate-type", "lib", "--crate-name", "verilog_parser",
               "--out-dir", RLIB_CACHE, parser_src.to_str().unwrap(), "--edition", "2021"])
        .output()
        .map_err(|e| format!("failed to spawn rustc: {}", e))?;
    if !output.status.success() {
        return Err(format!("verilog-parser compilation failed:\n{}",
            String::from_utf8_lossy(&output.stderr)));
    }

    let lib_src = std::path::Path::new(manifest)
        .join("../../verilog2rust/src/lib.rs");
    let output = Command::new("rustc")
        .args(["--crate-type", "lib", "--crate-name", "verilog2rust",
               "--out-dir", RLIB_CACHE,
               "--extern", &format!("verilog_parser={}", parser_rlib),
               "-L", RLIB_CACHE,
               lib_src.to_str().unwrap(), "--edition", "2021"])
        .output()
        .map_err(|e| format!("failed to spawn rustc: {}", e))?;
    if !output.status.success() {
        return Err(format!("verilog2rust library compilation failed:\n{}",
            String::from_utf8_lossy(&output.stderr)));
    }

    Ok(rlib_path)
}

fn handle_sim(code: String, top: Option<String>) -> Response {
    let _ = std::fs::create_dir_all(BIN_CACHE).map_err(|e| e.to_string());

    let temp_dir = std::env::temp_dir();
    let vpath = temp_dir.join(format!("verilog_{}_{}.v", std::process::id(), next_temp_id()));
    if let Err(e) = std::fs::write(&vpath, &code) {
        return Response::Error { error: format!("Failed to write verilog: {}", e) };
    }

    let vpath_str = vpath.to_str().unwrap();
    let modules = match catch_unwind(|| verilog2rust::parse_file(vpath_str)) {
        Ok(m) => m,
        Err(_) => return Response::Error { error: "Verilog parsing panicked".into() },
    };

    let generated_rust = match catch_unwind(|| verilog2rust::gen_ruhdl(&modules)) {
        Ok(r) => r,
        Err(_) => return Response::Error { error: "Rust code generation panicked".into() },
    };

    let _ = std::fs::create_dir_all("/tmp/verilog2rust_src").map_err(|e| e.to_string());
    let src_name = format!("design_{}", next_temp_id());
    let src_path = format!("/tmp/verilog2rust_src/{}.rs", src_name);
    if let Err(e) = std::fs::write(&src_path, &generated_rust) {
        return Response::Error { error: format!("Failed to write generated Rust: {}", e) };
    }

    let rlib_path = match build_lib_rlib() {
        Ok(p) => p,
        Err(e) => return Response::Error { error: e },
    };

    let rlib_dir = std::path::Path::new(RLIB_CACHE);
    let out_name = format!("{}_{}", src_name, std::process::id());
    let out_path = format!("{}/{}", BIN_CACHE, out_name);

    let parser_rlib = format!("{}/libverilog_parser.rlib", RLIB_CACHE);

    let mut cmd = Command::new("rustc");
    cmd.args([
        "--extern", &format!("verilog2rust={}", rlib_path),
        "--extern", &format!("verilog_parser={}", parser_rlib),
        "-L", rlib_dir.to_str().unwrap(),
        &src_path,
        "-o", &out_path,
        "--edition", "2021",
    ]);

    let compile_output = match cmd.output() {
        Ok(out) => out,
        Err(e) => return Response::VerilogSimResult {
            stdout: String::new(),
            stderr: format!("Failed to spawn rustc: {}", e),
            generated_rust,
        },
    };

    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr);
        let stdout = String::from_utf8_lossy(&compile_output.stdout);
        return Response::VerilogSimResult {
            stdout: stdout.to_string(),
            stderr: format!("Compilation failed:\n{}", stderr),
            generated_rust,
        };
    }

    use std::sync::mpsc;
    let child = match Command::new(&out_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            let _ = std::fs::remove_file(&out_path);
            return Response::VerilogSimResult {
                stdout: String::new(),
                stderr: format!("Failed to spawn simulation: {}\n", e),
                generated_rust,
            };
        }
    };

    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let output = child.wait_with_output();
        let _ = tx.send(output);
    });

    const TIMEOUT_SECS: u64 = 15;
    let (run_stdout, run_stderr) = match rx.recv_timeout(std::time::Duration::from_secs(TIMEOUT_SECS)) {
        Ok(Ok(out)) => (
            String::from_utf8_lossy(&out.stdout).to_string(),
            String::from_utf8_lossy(&out.stderr).to_string(),
        ),
        Ok(Err(e)) => (String::new(), format!("Execution failed: {}\n", e)),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            (String::new(), format!("Simulation timed out after {} seconds (infinite loop?)\n", TIMEOUT_SECS))
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            (String::new(), "Simulation process crashed\n".into())
        }
    };

    let _ = std::fs::remove_file(&out_path);

    Response::VerilogSimResult {
        stdout: run_stdout,
        stderr: run_stderr,
        generated_rust,
    }
}