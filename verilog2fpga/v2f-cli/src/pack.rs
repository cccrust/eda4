use std::fs;
use std::path::Path;
use std::process::Command;

use v2f_core::{Device, V2fError, V2fResult};

pub fn check_tool() -> bool {
    Command::new("icepack").arg("--version").output().is_ok()
}

/// 使用 icepack 打包
pub fn run_pack(asc: &Path, bin: &Path) -> V2fResult<()> {
    let status = Command::new("icepack")
        .arg(asc)
        .arg(bin)
        .status()
        .map_err(|_| V2fError::ToolNotFound("icepack".into()))?;

    if !status.success() {
        return Err(V2fError::PackFailed(format!(
            "icepack 執行失敗 (exit code: {:?})",
            status.code()
        )));
    }
    Ok(())
}

/// 純 Rust 打包（不依賴 icepack）
pub fn run_pack_pure(asc_path: &Path, bin_path: &Path, device: Device) -> V2fResult<()> {
    let src = fs::read_to_string(asc_path)
        .map_err(|e| V2fError::Io(e))?;

    let asc = v2f_bitstream::asc::parse_asc(&src).map_err(|e| {
        V2fError::PackFailed(format!("ASC 解析錯誤: {e}"))
    })?;

    let ice_dev = match device {
        Device::HX1K => v2f_db::ice40::Ice40Device::HX1K,
        Device::HX4K => v2f_db::ice40::Ice40Device::HX4K,
        Device::HX8K => v2f_db::ice40::Ice40Device::HX8K,
        Device::LP1K => v2f_db::ice40::Ice40Device::LP1K,
        Device::UP5K => v2f_db::ice40::Ice40Device::UP5K,
    };

    let mut cram = v2f_bitstream::Cram::new(ice_dev);
    v2f_bitstream::asc::apply_asc_to_cram(&asc, &mut cram, ice_dev);
    let bitstream = v2f_bitstream::pack::pack_bitstream(&cram);

    fs::write(bin_path, &bitstream)
        .map_err(|e| V2fError::Io(e))?;

    Ok(())
}
