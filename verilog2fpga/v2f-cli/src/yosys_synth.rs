use std::path::Path;
use std::process::Command;

use v2f_core::{Device, V2fError, V2fResult};

pub fn check_tool() -> bool {
    Command::new("yosys").arg("--version").output().is_ok()
}

pub fn run_synth(
    input: &Path,
    output: &Path,
    _device: Device,
    top: Option<&str>,
) -> V2fResult<()> {
    let top = top.unwrap_or("top");
    let script = format!("synth_ice40 -json {} -top {}", output.display(), top);

    let status = Command::new("yosys")
        .arg("-p")
        .arg(&script)
        .arg("-q")
        .arg(input)
        .status()
        .map_err(|_| V2fError::ToolNotFound("yosys".into()))?;

    if !status.success() {
        return Err(V2fError::SynthesisFailed(format!(
            "yosys 執行失敗 (exit code: {:?})",
            status.code()
        )));
    }
    Ok(())
}
