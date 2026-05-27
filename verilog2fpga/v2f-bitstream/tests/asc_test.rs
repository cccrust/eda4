use v2f_bitstream::asc::parse_asc;

#[test]
fn test_parse_empty_asc() {
    let asc = parse_asc(".module test\n.synckey 0xDEADBEEF\n").unwrap();
    assert_eq!(asc.module, "test");
    assert_eq!(asc.synckey, 0xDEADBEEF);
    assert!(asc.logic_tiles.is_empty());
    assert!(asc.io_tiles.is_empty());
}

#[test]
fn test_parse_minimal_asc() {
    let src = r#"
.module top
.io_tile 0 0
  .pad 0 clk
.logic_tile 1 1
  .lut 0 2 3 1 0 "0123"
  .wiring 0 0 0 128
.synckey 0x12345678
"#;
    let asc = parse_asc(src).unwrap();
    assert_eq!(asc.module, "top");
    assert_eq!(asc.synckey, 0x12345678);
    assert_eq!(asc.io_tiles.len(), 1);
    assert_eq!(asc.logic_tiles.len(), 1);

    let lt = &asc.logic_tiles[0];
    assert_eq!(lt.luts.len(), 1);
    assert_eq!(lt.luts[0].init, 0x0123);
    assert_eq!(lt.wiring.len(), 1);
}

#[test]
fn test_parse_comment_handling() {
    let src = ".module top\n# this is a comment\n.synckey 0x42\n";
    let asc = parse_asc(src).unwrap();
    assert_eq!(asc.synckey, 0x42);
}

#[test]
fn test_parse_empty_lines() {
    let src = ".module top\n\n\n.synckey 0x0\n";
    let asc = parse_asc(src).unwrap();
    assert_eq!(asc.synckey, 0x0);
}
