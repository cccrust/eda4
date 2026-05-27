use v2f_core::Device;

#[test]
fn test_device_parse() {
    assert_eq!("hx1k".parse::<Device>().unwrap(), Device::HX1K);
    assert_eq!("HX8K".parse::<Device>().unwrap(), Device::HX8K);
    assert_eq!("UP5K".parse::<Device>().unwrap(), Device::UP5K);
    assert_eq!("lp1k".parse::<Device>().unwrap(), Device::LP1K);
    assert_eq!("hx4k".parse::<Device>().unwrap(), Device::HX4K);
}

#[test]
fn test_device_invalid() {
    assert!("xyz".parse::<Device>().is_err());
    assert!("".parse::<Device>().is_err());
}

#[test]
fn test_device_names() {
    assert_eq!(Device::HX1K.name(), "hx1k");
    assert_eq!(Device::HX8K.name(), "hx8k");
    assert_eq!(Device::UP5K.name(), "up5k");
}

#[test]
fn test_device_nextpnr_flags() {
    assert_eq!(Device::HX1K.nextpnr_flag(), "--hx1k");
    assert_eq!(Device::HX8K.nextpnr_flag(), "--hx8k");
}

#[test]
fn test_device_all() {
    let all = Device::all();
    assert_eq!(all.len(), 5);
    assert!(all.contains(&Device::HX1K));
    assert!(all.contains(&Device::HX8K));
}

#[test]
fn test_device_display() {
    assert_eq!(format!("{}", Device::HX8K), "hx8k");
    assert_eq!(format!("{}", Device::UP5K), "up5k");
}
