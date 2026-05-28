use std::process::Command;
use v2f_bitstream::asc::{apply_asc_to_cram, parse_asc};
use v2f_bitstream::cram::Cram;
use v2f_bitstream::pack::{pack_bitstream, crc32, PREAMBLE_SIZE};
use v2f_db::ice40::Ice40Device;

const MINIMAL_ASC: &str = include_str!("../_fixtures/minimal_hx1k.asc");
const EMPTY_ASC: &str = include_str!("../_fixtures/empty_hx1k.asc");

fn run_icepack(asc_src: &str) -> Option<Vec<u8>> {
    if !Command::new("icepack").arg("--version").output().is_ok() {
        return None;
    }
    let tmp = std::env::temp_dir().join(format!("v2f_xv_pack_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).ok()?;
    let asc_path = tmp.join("input.asc");
    let bin_path = tmp.join("output.bin");
    std::fs::write(&asc_path, asc_src).ok()?;
    let status = Command::new("icepack")
        .args([asc_path.to_str().unwrap(), bin_path.to_str().unwrap()])
        .status()
        .ok()?;
    if !status.success() {
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
    let pure_bin = pack_pure(MINIMAL_ASC, Ice40Device::HX1K);
    if let Some(icepack_bin) = run_icepack(MINIMAL_ASC) {
        assert_eq!(
            pure_bin, icepack_bin,
            "binary identical to icepack for minimal ASC"
        );
    }
}

#[test]
fn cross_pack_empty_vs_icepack() {
    let pure_bin = pack_pure(EMPTY_ASC, Ice40Device::HX1K);
    assert!(!pure_bin.is_empty());
    if let Some(icepack_bin) = run_icepack(EMPTY_ASC) {
        assert_eq!(
            pure_bin, icepack_bin,
            "binary identical to icepack for empty ASC"
        );
    }
}

#[test]
fn cross_pack_deterministic_minimal() {
    let a = pack_pure(MINIMAL_ASC, Ice40Device::HX1K);
    let b = pack_pure(MINIMAL_ASC, Ice40Device::HX1K);
    assert_eq!(a, b, "pack must be deterministic");
}
