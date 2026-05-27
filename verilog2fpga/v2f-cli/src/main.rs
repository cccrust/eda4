mod pack;
mod pnr;
mod prog;
mod yosys_synth;

use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use v2f_core::{Device, V2fResult};

#[derive(Parser)]
#[command(name = "v2f", about = "Verilog → FPGA 工具鏈 (v0.5)")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 完整流程: 綜合 → 佈局佈線 → 打包
    Build {
        input: PathBuf,
        #[arg(long, default_value = "hx8k")]
        device: String,
        #[arg(long)]
        pcf: Option<PathBuf>,
        #[arg(long)]
        top: Option<String>,
        #[arg(long, default_value = "output")]
        output: String,
        #[arg(long, default_value_t = String::from("auto"))]
        backend: String,
        #[arg(long, default_value_t = String::from("verilog"))]
        lang: String,
    },
    /// 邏輯綜合
    Synth {
        input: PathBuf,
        #[arg(long, default_value = "output.json")]
        output: PathBuf,
        #[arg(long, default_value = "hx8k")]
        device: String,
        #[arg(long)]
        top: Option<String>,
        #[arg(long, default_value_t = String::from("auto"))]
        backend: String,
    },
    /// 佈局佈線
    Pnr {
        input: PathBuf,
        #[arg(long, default_value = "output.asc")]
        output: PathBuf,
        #[arg(long, default_value = "hx8k")]
        device: String,
        #[arg(long)]
        pcf: Option<PathBuf>,
        #[arg(long, default_value_t = String::from("auto"))]
        backend: String,
    },
    /// 打包位元流
    Pack {
        input: PathBuf,
        #[arg(long, default_value = "output.bin")]
        output: PathBuf,
        #[arg(long, default_value_t = String::from("auto"))]
        backend: String,
    },
    /// 燒錄至 FPGA
    Prog {
        input: PathBuf,
        #[arg(long, default_value_t = String::from("auto"))]
        driver: String,
    },
    /// 列出支援的裝置
    ListDevices,
    /// 檢查外部工具是否可用
    Check,
}

fn main() -> V2fResult<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Build {
            input,
            device,
            pcf,
            top,
            output,
            backend,
            lang,
        } => {
            let dev: Device = device
                .parse()
                .map_err(|e| v2f_core::V2fError::Config(e))?;
            let json_path = PathBuf::from(format!("{}.json", output));
            let asc_path = PathBuf::from(format!("{}.asc", output));
            let bin_path = PathBuf::from(format!("{}.bin", output));

            match lang.as_str() {
                "rust" => {
                    let module = v2f_rust::HdlModule::new(top.as_deref().unwrap_or("top"))
                        .input("clk", 1)
                        .output("led", 1)
                        .reg("counter", 26)
                        .dff("counter", v2f_rust::HdlExpr::Add(
                            Box::new(v2f_rust::HdlExpr::Ident("counter".into())),
                            Box::new(v2f_rust::HdlExpr::Const(1, 26)),
                        ))
                        .assign("led", v2f_rust::HdlExpr::Index(
                            Box::new(v2f_rust::HdlExpr::Ident("counter".into())), 25,
                        ));
                    let json = v2f_rust::compile(&module);
                    fs::write(&json_path, &json)
                        .map_err(|e| v2f_core::V2fError::Io(e))?;
                    pure_pnr(&json_path, &asc_path, dev)?;
                    pack::run_pack_pure(&asc_path, &bin_path, dev)?;
                }
                _ => {
                    match backend.as_str() {
                        "yosys" | "auto" => {
                            if backend == "auto" && !yosys_synth::check_tool() {
                                return Err(v2f_core::V2fError::ToolNotFound("yosys".into()));
                            }
                            yosys_synth::run_synth(&input, &json_path, dev, top.as_deref())?;
                            if pnr::check_tool() {
                                pnr::run_pnr(&json_path, &asc_path, dev, pcf.as_deref())?;
                            } else {
                                return Err(v2f_core::V2fError::ToolNotFound("nextpnr".into()));
                            }
                            pack::run_pack(&asc_path, &bin_path)?;
                        }
                        "pnr-only" => {
                            if !yosys_synth::check_tool() {
                                return Err(v2f_core::V2fError::ToolNotFound("yosys".into()));
                            }
                            yosys_synth::run_synth(&input, &json_path, dev, top.as_deref())?;
                            pure_pnr(&json_path, &asc_path, dev)?;
                            pack::run_pack_pure(&asc_path, &bin_path, dev)?;
                        }
                        "pure-rust" | "rust" => {
                            pure_synth(&input, &json_path, top.as_deref())?;
                            pure_pnr(&json_path, &asc_path, dev)?;
                            pack::run_pack_pure(&asc_path, &bin_path, dev)?;
                        }
                        _ => {
                            return Err(v2f_core::V2fError::Config(format!(
                                "未知 backend: {backend}。支援: auto, yosys, pnr-only, pure-rust"
                            )));
                        }
                    }
                }
            }
            println!("✓ 綜合完成: {}", json_path.display());
            println!("✓ 佈局佈線完成: {}", asc_path.display());
            println!("✓ 位元流打包完成: {}", bin_path.display());
        }
        Command::Synth {
            input,
            output,
            device: _,
            top,
            backend,
        } => {
            match backend.as_str() {
                "rust" => pure_synth(&input, &output, top.as_deref())?,
                "yosys" | "auto" => {
                    if yosys_synth::check_tool() {
                        let dev = Device::HX8K;
                        yosys_synth::run_synth(&input, &output, dev, top.as_deref())?;
                    } else if backend == "auto" {
                        pure_synth(&input, &output, top.as_deref())?;
                    } else {
                        return Err(v2f_core::V2fError::ToolNotFound("yosys".into()));
                    }
                }
                _ => {
                    return Err(v2f_core::V2fError::Config(format!(
                        "未知 backend: {backend}。支援: auto, rust, yosys"
                    )));
                }
            }
            println!("✓ 綜合完成: {}", output.display());
        }
        Command::Pnr {
            input,
            output,
            device,
            pcf: _,
            backend,
        } => {
            let dev: Device = device
                .parse()
                .map_err(|e| v2f_core::V2fError::Config(e))?;
            match backend.as_str() {
                "rust" | "pure-rust" | "pnr-only" => pure_pnr(&input, &output, dev)?,
                "yosys" | "auto" => {
                    if pnr::check_tool() {
                        let pcf_path: Option<&std::path::Path> = None;
                        pnr::run_pnr(&input, &output, dev, pcf_path)?;
                    } else {
                        pure_pnr(&input, &output, dev)?;
                    }
                }
                _ => {
                    return Err(v2f_core::V2fError::Config(format!(
                        "未知 backend: {backend}。支援: auto, yosys, pnr-only, pure-rust"
                    )));
                }
            }
            println!("✓ 佈局佈線完成: {}", output.display());
        }
        Command::Pack {
            input,
            output,
            backend,
        } => {
            match backend.as_str() {
                "auto" => {
                    if pack::check_tool() {
                        pack::run_pack(&input, &output)?;
                    } else {
                        let dev = Device::HX8K;
                        pack::run_pack_pure(&input, &output, dev)?;
                    }
                }
                "rust" => {
                    let dev = Device::HX8K;
                    pack::run_pack_pure(&input, &output, dev)?;
                }
                "icepack" => {
                    pack::run_pack(&input, &output)?;
                }
                _ => {
                    return Err(v2f_core::V2fError::Config(format!(
                        "未知 backend: {backend}。支援: auto, rust, icepack"
                    )));
                }
            }
            println!("✓ 位元流打包完成: {}", output.display());
        }
        Command::Prog { input, driver } => {
            match driver.as_str() {
                "mock" => {
                    let bs = fs::read(&input).map_err(|e| v2f_core::V2fError::Io(e))?;
                    let mut jtag = v2f_programmer::jtag::JtagStateMachine::new();
                    v2f_programmer::Ice40Programmer::program_cram_jtag(&mut jtag, &bs)
                        .map_err(|e| v2f_core::V2fError::Config(e))?;
                    println!("✓ 模擬燒錄完成 (JTAG, {} bytes)", bs.len());
                }
                "spi" => {
                    let bs = fs::read(&input).map_err(|e| v2f_core::V2fError::Io(e))?;
                    let mut flash = v2f_programmer::spi::SpiFlash::new(bs.len().max(4096));
                    v2f_programmer::Ice40Programmer::program_spi_flash(&bs, &mut flash)
                        .map_err(|e| v2f_core::V2fError::Config(e))?;
                    if v2f_programmer::Ice40Programmer::verify_flash(&bs, &flash) {
                        println!("✓ 模擬 SPI 燒錄完成 ({}, {} bytes)", input.display(), bs.len());
                    } else {
                        return Err(v2f_core::V2fError::Config("SPI 驗證失敗".into()));
                    }
                }
                _ => {
                    prog::run_prog(&input)?;
                    println!("✓ 燒錄完成");
                }
            }
        }
        Command::ListDevices => {
            println!("支援的 iCE40 裝置:");
            for dev in Device::all() {
                println!("  {dev}");
            }
        }
        Command::Check => {
            let checks: [(&str, bool); 9] = [
                ("yosys", yosys_synth::check_tool()),
                ("nextpnr-ice40", pnr::check_tool()),
                ("icepack", pack::check_tool()),
                ("openFPGALoader/iceprog", prog::check_tool()),
                ("v2f-synth (pure Rust)", true),
                ("v2f-pnr (pure Rust)", true),
                ("v2f-programmer (mock)", true),
                ("v2f-rust (HDL bridge)", true),
                ("v2f-bitstream (pure Rust)", true),
            ];
            for (name, ok) in &checks {
                let mark = if *ok { "✓" } else { "✗" };
                let installed = if *ok { "已安裝" } else { "未安裝" };
                println!("  {mark} {name}: {installed}");
            }
        }
    }

    Ok(())
}

fn pure_synth(input: &PathBuf, output: &PathBuf, top: Option<&str>) -> V2fResult<()> {
    let src = fs::read_to_string(input).map_err(|e| v2f_core::V2fError::Io(e))?;
    let top_name = top.unwrap_or("top");
    let json = v2f_synth::synthesize(&src, top_name);
    fs::write(output, &json).map_err(|e| v2f_core::V2fError::Io(e))?;
    Ok(())
}

fn pure_pnr(json_path: &PathBuf, asc_path: &PathBuf, dev: Device) -> V2fResult<()> {
    let json_str =
        fs::read_to_string(json_path).map_err(|e| v2f_core::V2fError::Io(e))?;
    let asc = v2f_pnr::run_pnr(&json_str, dev);
    fs::write(asc_path, &asc).map_err(|e| v2f_core::V2fError::Io(e))?;
    Ok(())
}
