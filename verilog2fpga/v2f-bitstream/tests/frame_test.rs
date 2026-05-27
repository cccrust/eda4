use v2f_bitstream::frame::Frame;

#[test]
fn test_frame_new_is_zeroed() {
    let f = Frame::new();
    assert_eq!(f.as_bytes(), &[0u8; 165]);
}

#[test]
fn test_frame_set_get_bit() {
    let mut f = Frame::new();
    assert!(!f.get_bit(0));
    assert!(!f.get_bit(1319));

    f.set_bit(0);
    assert!(f.get_bit(0));

    f.set_bit(1319);
    assert!(f.get_bit(1319));
}

#[test]
fn test_frame_clear_bit() {
    let mut f = Frame::new();
    f.set_bit(42);
    assert!(f.get_bit(42));
    f.clear_bit(42);
    assert!(!f.get_bit(42));
}

#[test]
fn test_frame_words() {
    let mut f = Frame::new();
    f.set_word(0, 0xAABBCCDDEE);
    assert_eq!(f.get_word(0), 0xAABBCCDDEE);

    f.set_word(32, 0x123456789A);
    assert_eq!(f.get_word(32), 0x123456789A);
}

#[test]
fn test_frame_bytes_roundtrip() {
    let original = Frame::new();
    let bytes = *original.as_bytes();
    let restored = Frame::from_bytes(&bytes);
    assert_eq!(original.as_bytes(), restored.as_bytes());
}

#[test]
fn test_frame_bit_independence() {
    let mut f = Frame::new();
    for i in 0..1320 {
        f.set_bit(i);
        assert!(f.get_bit(i));
        f.clear_bit(i);
        assert!(!f.get_bit(i));
    }
}
