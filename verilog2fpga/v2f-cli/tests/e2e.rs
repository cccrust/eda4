use std::process::Command;

#[test]
fn test_list_devices() {
    let output = Command::new(env!("CARGO_BIN_EXE_v2f"))
        .arg("list-devices")
        .output()
        .expect("failed to run v2f list-devices");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hx1k"));
    assert!(stdout.contains("hx8k"));
    assert!(stdout.contains("up5k"));
    assert!(stdout.contains("lp1k"));
    assert!(stdout.contains("hx4k"));
}

#[test]
fn test_check_never_panics() {
    let output = Command::new(env!("CARGO_BIN_EXE_v2f"))
        .arg("check")
        .output()
        .expect("failed to run v2f check");
    assert!(output.status.success());
}
