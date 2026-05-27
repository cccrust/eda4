use std::path::Path;
use std::process::Command;

use v2f_core::{Device, V2fError, V2fResult};

pub fn check_tool() -> bool {
    let variants = ["nextpnr-ice40", "nextpnr"];
    variants
        .iter()
        .any(|cmd| Command::new(cmd).arg("--version").output().is_ok())
}

pub fn run_pnr(
    json: &Path,
    asc: &Path,
    device: Device,
    pcf: Option<&Path>,
) -> V2fResult<()> {
    let tool = if Command::new("nextpnr-ice40")
        .arg("--version")
        .output()
        .is_ok()
    {
        "nextpnr-ice40"
    } else {
        "nextpnr"
    };

    let mut cmd = Command::new(tool);
    cmd.arg(device.nextpnr_flag())
        .arg("--json")
        .arg(json)
        .arg("--asc")
        .arg(asc);

    if let Some(pcf_path) = pcf {
        cmd.arg("--pcf").arg(pcf_path);
    }

    let status = cmd
        .status()
        .map_err(|_| V2fError::ToolNotFound(tool.into()))?;

    if !status.success() {
        return Err(V2fError::PnrFailed(format!(
            "{} 執行失敗 (exit code: {:?})",
            tool,
            status.code()
        )));
    }
    Ok(())
}
