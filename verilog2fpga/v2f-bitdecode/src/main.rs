use std::fs;
use std::path::PathBuf;
use clap::Parser;

use v2f_bitdecode::bin_parse::parse_bin;
use v2f_bitdecode::cram_decode::decode_cram;
use v2f_bitdecode::json_out::tiles_to_json;

#[derive(Parser)]
#[command(name = "v2f-bitdecode", about = "iCE40 bitstream decoder — BIN → JSON")]
struct Cli {
    /// Input BIN file
    input: PathBuf,

    /// Output JSON file (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Pretty-print JSON (default: compact)
    #[arg(short, long)]
    pretty: bool,
}

fn main() {
    let cli = Cli::parse();
    let data = fs::read(&cli.input).unwrap_or_else(|e| {
        eprintln!("Error: cannot read {}: {}", cli.input.display(), e);
        std::process::exit(1);
    });
    let bin = match parse_bin(&data) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error: {}: {}", cli.input.display(), e);
            std::process::exit(1);
        }
    };
    let tiles = decode_cram(&bin.cram, bin.device);
    let json = tiles_to_json(&tiles, bin.device, cli.input.to_str().unwrap(), bin.crc_valid);
    let output = if cli.pretty {
        serde_json::to_string_pretty(&json)
    } else {
        serde_json::to_string(&json)
    }
    .unwrap_or_else(|e| {
        eprintln!("Error: JSON serialization failed: {}", e);
        std::process::exit(1);
    });
    if let Some(path) = cli.output {
        fs::write(&path, &output).unwrap_or_else(|e| {
            eprintln!("Error: cannot write {}: {}", path.display(), e);
            std::process::exit(1);
        });
    } else {
        println!("{}", output);
    }
}
