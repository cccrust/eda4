//! BIN layer cross-validation tests.
//!
//! 注意：v2f-bitstream 和 icepack 使用不同的 ASC 格式和不同的 BIN 包裝格式，
//! 因此無法直接 byte-by-byte 比較。本檔案僅做內部一致性驗證。
//! - v2f-bitstream ASC: `.module`/`.synckey` 格式 → CRAM BIN (32 zero preamble + CRC)
//! - icepack ASC: `.device 1k` 格式（無 `.module`/`.synckey`）→ iCE BIN (JEDEC-like)
//!
//! 未來若需冰封比對，需統一格式或實作格式轉換。

use std::process::Command;
use v2f_bitstream::asc::{apply_asc_to_cram, parse_asc};
use v2f_bitstream::cram::Cram;
use v2f_bitstream::frame::FRAME_BITS;
use v2f_bitstream::pack::{pack_bitstream, crc32, PREAMBLE_SIZE};
use v2f_db::ice40::Ice40Device;

const MINIMAL_ASC: &str = include_str!("../_fixtures/minimal_hx1k.asc");
const EMPTY_ASC: &str = include_str!("../_fixtures/empty_hx1k.asc");

/// 嘗試執行 icepack。僅在 icepack 存在且 ASC 格式正確時回傳 Some。
fn run_icepack(asc_src: &str) -> Option<Vec<u8>> {
    // icepack 不支援 --version，改用 which 檢查是否存在
    if Command::new("which").arg("icepack").output().is_err() {
        return None;
    }
    let tmp = std::env::temp_dir().join(format!("v2f_xv_pack_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).ok()?;
    let asc_path = tmp.join("input.asc");
    let bin_path = tmp.join("output.bin");
    std::fs::write(&asc_path, asc_src).ok()?;
    let output = Command::new("icepack")
        .args([asc_path.to_str().unwrap(), bin_path.to_str().unwrap()])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let result = std::fs::read(&bin_path).ok()?;
    let _ = std::fs::remove_dir_all(&tmp);
    Some(result)
}

fn pack_pure(asc_src: &str, device: Ice40Device) -> Vec<u8> {
    let asc = parse_asc(asc_src).expect("parse ASC");
    let mut cram = Cram::new(device);
    apply_asc_to_cram(&asc, &mut cram, device);
    pack_bitstream(&cram)
}

#[test]
fn cross_pack_minimal_well_formed() {
    let bin = pack_pure(MINIMAL_ASC, Ice40Device::HX1K);
    assert!(!bin.is_empty(), "bitstream should not be empty");
    assert_eq!(&bin[..PREAMBLE_SIZE], &[0u8; PREAMBLE_SIZE], "preamble");
    let total_bits = u32::from_le_bytes(bin[PREAMBLE_SIZE..PREAMBLE_SIZE + 4].try_into().unwrap());
    assert_eq!(
        total_bits,
        Ice40Device::HX1K.total_frames() * 1320
    );
    let crc_pos = bin.len() - 4;
    let cram_data = &bin[PREAMBLE_SIZE + 4..crc_pos];
    let stored_crc = u32::from_le_bytes(bin[crc_pos..crc_pos + 4].try_into().unwrap());
    assert_eq!(stored_crc, crc32(cram_data), "CRC mismatch");
}

#[test]
fn cross_pack_minimal_vs_icepack() {
    let _pure_bin = pack_pure(MINIMAL_ASC, Ice40Device::HX1K);
    // icepack 不吃 `.module`/`.synckey` 格式，此處僅檢查 icepack 存在
    // 若需真正的 binary 比對，需產生 icepack 相容格式的 ASC
    if let Some(_icepack_bin) = run_icepack(MINIMAL_ASC) {
        // icepack 產生的 BIN 格式不同（JEDEC-like vs CRAM），無法直接比對
        // TODO(v0.9): 實作格式轉換後啟用 binary 比對
    }
}

#[test]
fn cross_pack_empty_well_formed() {
    let bin = pack_pure(EMPTY_ASC, Ice40Device::HX1K);
    assert!(!bin.is_empty());
    let crc_pos = bin.len() - 4;
    let cram_data = &bin[PREAMBLE_SIZE + 4..crc_pos];
    let stored_crc = u32::from_le_bytes(bin[crc_pos..crc_pos + 4].try_into().unwrap());
    assert_eq!(stored_crc, crc32(cram_data), "CRC mismatch");
}

#[test]
fn cross_pack_deterministic_minimal() {
    let a = pack_pure(MINIMAL_ASC, Ice40Device::HX1K);
    let b = pack_pure(MINIMAL_ASC, Ice40Device::HX1K);
    assert_eq!(a, b, "pack must be deterministic");
}

#[test]
fn cross_pack_minimal_size_matches_hx1k() {
    let bin = pack_pure(MINIMAL_ASC, Ice40Device::HX1K);
    let expected_cram_bytes =
        Ice40Device::HX1K.total_frames() as usize * FRAME_BITS / 8;
    let expected_total = PREAMBLE_SIZE + 4 + expected_cram_bytes + 4;
    assert_eq!(bin.len(), expected_total, "total size mismatch");
}
