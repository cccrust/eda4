use std::path::Path;
use std::process::Command;

use v2f_core::{V2fError, V2fResult};

pub fn check_tool() -> bool {
    let variants = ["openFPGALoader", "iceprog"];
    variants
        .iter()
        .any(|cmd| Command::new(cmd).arg("--version").output().is_ok())
}

pub fn run_prog(bin: &Path) -> V2fResult<()> {
    if Command::new("openFPGALoader")
        .arg("--version")
        .output()
        .is_ok()
    {
        let status = Command::new("openFPGALoader")
            .arg("-b")
            .arg("ice40_generic")
            .arg(bin)
            .status()
            .map_err(|_| V2fError::ToolNotFound("openFPGALoader".into()))?;

        if !status.success() {
            return Err(V2fError::ProgFailed(format!(
                "openFPGALoader 執行失敗 (exit code: {:?})",
                status.code()
            )));
        }
    } else if Command::new("iceprog").arg("--version").output().is_ok() {
        let status = Command::new("iceprog")
            .arg(bin)
            .status()
            .map_err(|_| V2fError::ToolNotFound("iceprog".into()))?;

        if !status.success() {
            return Err(V2fError::ProgFailed(format!(
                "iceprog 執行失敗 (exit code: {:?})",
                status.code()
            )));
        }
    } else {
        return Err(V2fError::ToolNotFound(
            "openFPGALoader 或 iceprog".into(),
        ));
    }
    Ok(())
}
