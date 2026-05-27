use v2f_bitstream::cram::Cram;
use v2f_bitstream::pack::{pack_bitstream, crc32, PREAMBLE_SIZE};
use v2f_db::ice40::Ice40Device;

#[test]
fn test_pack_empty_cram_hx1k() {
    let dev = Ice40Device::HX1K;
    let cram = Cram::new(dev);
    let bs = pack_bitstream(&cram);

    // preamble
    assert_eq!(&bs[..PREAMBLE_SIZE], &[0u8; PREAMBLE_SIZE]);

    // bit count
    let total_bits =
        u32::from_le_bytes(bs[PREAMBLE_SIZE..PREAMBLE_SIZE + 4].try_into().unwrap());
    assert_eq!(total_bits, dev.total_frames() * 1320);
}

#[test]
fn test_pack_crc_valid() {
    let dev = Ice40Device::HX8K;
    let cram = Cram::new(dev);
    let bs = pack_bitstream(&cram);

    let crc_pos = bs.len() - 4;
    let cram_data = &bs[PREAMBLE_SIZE + 4..crc_pos];
    let stored_crc = u32::from_le_bytes(bs[crc_pos..crc_pos + 4].try_into().unwrap());
    let expected_crc = crc32(cram_data);
    assert_eq!(stored_crc, expected_crc, "CRC mismatch");
}

#[test]
fn test_pack_deterministic() {
    let dev = Ice40Device::UP5K;
    let a = pack_bitstream(&Cram::new(dev));
    let b = pack_bitstream(&Cram::new(dev));
    assert_eq!(a, b, "bitstream should be deterministic");
}

#[test]
fn test_lp1k_eq_hx1k_size() {
    let lp1k = pack_bitstream(&Cram::new(Ice40Device::LP1K)).len();
    let hx1k = pack_bitstream(&Cram::new(Ice40Device::HX1K)).len();
    // LP1K 與 HX1K 為相同晶粒，CRAM 大小應一致
    assert_eq!(lp1k, hx1k);
}
