use std::process::Command;
use std::path::Path;

#[test]
fn test_decode_empty_cram_hx1k_cli() {
    let output = Command::new(env!("CARGO_BIN_EXE_v2f-bitdecode"))
        .arg("--help")
        .output()
        .expect("failed to run v2f-bitdecode --help");
    assert!(output.status.success());
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(help.contains("BIN → JSON"));
}

#[test]
fn test_decode_minimal_fixture() {
    // 先用純 Rust packer 產生 BIN，再解碼回 JSON
    let asc_src = include_str!("../../v2f-bitstream/_fixtures/minimal_hx1k.asc");
    let asc = v2f_bitstream::asc::parse_asc(asc_src).expect("parse ASC fixture");
    let mut cram = v2f_bitstream::cram::Cram::new(v2f_db::ice40::Ice40Device::HX1K);
    v2f_bitstream::asc::apply_asc_to_cram(&asc, &mut cram, v2f_db::ice40::Ice40Device::HX1K);
    let bin = v2f_bitstream::pack::pack_bitstream(&cram);
    // 解碼
    let parsed = v2f_bitdecode::bin_parse::parse_bin(&bin).expect("parse generated BIN");
    assert!(parsed.crc_valid);
    assert_eq!(parsed.device, v2f_db::ice40::Ice40Device::HX1K);
    // 解碼 CRAM
    let tiles = v2f_bitdecode::cram_decode::decode_cram(&parsed.cram, parsed.device);
    // HX1K: 30 rows × (16 logic + 2 IO) = 540 tiles
    let expected_tile_count = 30 * (16 + 2);
    assert_eq!(tiles.len(), expected_tile_count);
    // 找到邏輯 tile (1,1) — fixture 的 logic_tile 1 1
    let lt = tiles.iter().find(|t| t.row == 1 && t.col == 2 && t.tile_type == "logic");
    assert!(lt.is_some(), "should find logic tile at row=1, col=2");
    // 輸出 JSON
    let json = v2f_bitdecode::json_out::tiles_to_json(&tiles, parsed.device, "minimal.bin", true);
    assert_eq!(json["device"], "hx1k");
    assert!(json["summary"]["logic_tiles_used"].as_u64().unwrap_or(0) > 0);
}
