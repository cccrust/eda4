use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

const LIB_SRC: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs");
const CACHE_DIR: &str = "/tmp/verilog2rust_rlib";

fn build_lib_rlib() -> String {
    fs::create_dir_all(CACHE_DIR).ok();
    let rlib_path = format!("{}/libverilog2rust.rlib", CACHE_DIR);
    if !Path::new(&rlib_path).exists() {
        let status = Command::new("rustc")
            .args(["--crate-type", "lib", "--crate-name", "verilog2rust",
                   "--out-dir", CACHE_DIR, LIB_SRC, "--edition", "2021"])
            .status()
            .expect("failed to build verilog2rust library");
        if !status.success() {
            panic!("verilog2rust library compilation failed");
        }
    }
    rlib_path
}

fn run_rhdl(input_path: &str) {
    let rlib = build_lib_rlib();
    let stem = Path::new(input_path).file_stem().and_then(|s| s.to_str()).unwrap_or("a.out");
    fs::create_dir_all("/tmp/verilog2rust_bin").ok();
    let binary = format!("/tmp/verilog2rust_bin/{}", stem);
    let status = Command::new("rustc")
        .args(["--extern", &format!("verilog2rust={}", rlib), input_path,
               "-o", &binary, "--edition", "2021"])
        .status()
        .expect("failed to compile ruHDL source");
    if !status.success() {
        panic!("compilation failed");
    }
    let mut child = Command::new(&binary)
        .spawn()
        .expect("failed to execute");
    child.wait().ok();
}

fn convert_verilog(input_path: &str, output_arg: &str) {
    let modules = verilog2rust::parse_file(input_path);
    let rust_code = verilog2rust::gen_ruhdl(&modules);
    let output_path = if output_arg.ends_with(".rs") || output_arg.ends_with(".rhdl") {
        output_arg.to_string()
    } else {
        let input_stem = Path::new(input_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        format!("{}/{}.rs", output_arg, input_stem)
    };
    if let Some(parent) = Path::new(&output_path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).ok();
        }
    }
    fs::write(&output_path, &rust_code)
        .expect(&format!("Failed to write output: {}", output_path));
    println!("Generated: {}", output_path);
    for m in &modules {
        println!("  module: {}", m.name);
        println!("    ports: {}", m.ports.iter().map(|p| format!("{}: {:?}", p.name, p.direction)).collect::<Vec<_>>().join(", "));
        println!("    items: {}", m.items.len());
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage:");
        println!("  verilog2rust <input.v> [output.rs]    Convert Verilog to Rust (ruHDL)");
        println!("  verilog2rust <input.rhdl>              Compile and run ruHDL");
        println!();
        println!("Examples:");
        println!("  verilog2rust adder4_tb.v");
        println!("  verilog2rust adder4_tb.v adder4_tb.rs");
        println!("  verilog2rust adder4_tb.rhdl");
        return;
    }

    let input_path = &args[1];

    if input_path.ends_with(".v") {
        if args.len() > 2 {
            convert_verilog(input_path, &args[2]);
        } else {
            convert_verilog(input_path, ".");
        }
    } else if input_path.ends_with(".rhdl") || input_path.ends_with(".rs") {
        run_rhdl(input_path);
    } else {
        eprintln!("Unknown file extension (use .v, .rhdl, or .rs)");
    }
}
