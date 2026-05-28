use std::process::Command;

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
    let dev = v2f_db::ice40::Ice40Device::HX1K;
    let asc_src = include_str!("../../v2f-bitstream/_fixtures/minimal_hx1k.asc");
    let asc = v2f_bitstream::asc::parse_asc(asc_src).expect("parse ASC fixture");
    let mut cram = v2f_bitstream::cram::Cram::new(dev);
    v2f_bitstream::asc::apply_asc_to_cram(&asc, &mut cram, dev);

    // 驗證 CRAM 有寫入：tile_start_frame(1,1,Logic) = 1*118 + 1*7 = 125
    let fpr = dev.frames_per_row();
    let start = 1 * fpr + 1 * 7;
    let f0 = cram.get_frame(start);
    assert_ne!(f0.get_word(0), 0, "frame {} word 0 should have bit set", start);

    let bin = v2f_bitstream::pack::pack_bitstream(&cram);
    let parsed = v2f_bitdecode::bin_parse::parse_bin(&bin).expect("parse generated BIN");
    assert!(parsed.crc_valid);
    assert_eq!(parsed.device, dev);

    let tiles = v2f_bitdecode::cram_decode::decode_cram(&parsed.cram, parsed.device);
    let expected_tile_count = (dev.num_rows() * (dev.num_cols() + 2)) as usize;
    assert_eq!(tiles.len(), expected_tile_count);

    // tile_start_frame(1,1,Logic) = 125, decode reads same range
    // decode col = physical_col + 1, so physical col=1 → decode col=2
    let lt = tiles.iter().find(|t| t.row == 1 && t.col == 2 && t.tile_type == "logic");
    assert!(lt.is_some(), "should find logic tile at row=1, col=2");
    if let Some(t) = lt {
        assert!(!t.non_zero_words.is_empty(), "logic tile (1,2) should have non-zero words");
    }

    let json = v2f_bitdecode::json_out::tiles_to_json(&tiles, parsed.device, "minimal.bin", true);
    assert_eq!(json["device"], "hx1k");
    assert!(
        json["summary"]["logic_tiles_used"].as_u64().unwrap_or(0) > 0,
        "should have at least 1 used logic tile"
    );
}
