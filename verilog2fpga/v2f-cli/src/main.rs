mod pack;
mod pnr;
mod prog;
mod synth;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use v2f_core::{Device, V2fResult};

#[derive(Parser)]
#[command(name = "v2f", about = "Verilog → FPGA 工具鏈 (v0.2)")]
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
        #[arg(long, default_value_t = false)]
        rust: bool,
    },
    /// 邏輯綜合 (Yosys)
    Synth {
        input: PathBuf,
        #[arg(long, default_value = "output.json")]
        output: PathBuf,
        #[arg(long, default_value = "hx8k")]
        device: String,
        #[arg(long)]
        top: Option<String>,
    },
    /// 佈局佈線 (nextpnr)
    Pnr {
        input: PathBuf,
        #[arg(long, default_value = "output.asc")]
        output: PathBuf,
        #[arg(long, default_value = "hx8k")]
        device: String,
        #[arg(long)]
        pcf: Option<PathBuf>,
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
            rust,
        } => {
            let dev: Device = device
                .parse()
                .map_err(|e| v2f_core::V2fError::Config(e))?;
            let json_path = PathBuf::from(format!("{}.json", output));
            let asc_path = PathBuf::from(format!("{}.asc", output));
            let bin_path = PathBuf::from(format!("{}.bin", output));

            synth::run_synth(&input, &json_path, dev, top.as_deref())?;
            println!("✓ 綜合完成: {}", json_path.display());

            pnr::run_pnr(&json_path, &asc_path, dev, pcf.as_deref())?;
            println!("✓ 佈局佈線完成: {}", asc_path.display());

            if rust {
                pack::run_pack_pure(&asc_path, &bin_path, dev)?;
            } else {
                pack::run_pack(&asc_path, &bin_path)?;
            }
            println!("✓ 位元流打包完成: {}", bin_path.display());

            println!(
                "完整流程完成。執行 'v2f prog {}' 燒錄。",
                bin_path.display()
            );
        }
        Command::Synth {
            input,
            output,
            device,
            top,
        } => {
            let dev: Device = device
                .parse()
                .map_err(|e| v2f_core::V2fError::Config(e))?;
            synth::run_synth(&input, &output, dev, top.as_deref())?;
            println!("✓ 綜合完成: {}", output.display());
        }
        Command::Pnr {
            input,
            output,
            device,
            pcf,
        } => {
            let dev: Device = device
                .parse()
                .map_err(|e| v2f_core::V2fError::Config(e))?;
            pnr::run_pnr(&input, &output, dev, pcf.as_deref())?;
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
        Command::Prog { input } => {
            prog::run_prog(&input)?;
            println!("✓ 燒錄完成");
        }
        Command::ListDevices => {
            println!("支援的 iCE40 裝置:");
            for dev in Device::all() {
                println!("  {dev}");
            }
        }
        Command::Check => {
            let checks: [(&str, bool); 4] = [
                ("yosys", synth::check_tool()),
                ("nextpnr-ice40", pnr::check_tool()),
                ("icepack", pack::check_tool()),
                ("openFPGALoader/iceprog", prog::check_tool()),
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
