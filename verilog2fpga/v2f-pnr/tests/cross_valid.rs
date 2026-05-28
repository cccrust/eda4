use std::process::Command;
use v2f_core::Device;

fn run_nextpnr(json_str: &str, device: Device) -> Option<String> {
    if !Command::new("nextpnr-ice40").arg("--version").output().is_ok() {
        return None;
    }
    let tmp = std::env::temp_dir().join(format!("v2f_xv_pnr_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).ok()?;
    let json_path = tmp.join("input.json");
    let asc_path = tmp.join("output.asc");
    std::fs::write(&json_path, json_str).ok()?;
    let status = Command::new("nextpnr-ice40")
        .args([
            device.nextpnr_flag(),
            "--json",
            json_path.to_str().unwrap(),
            "--asc",
            asc_path.to_str().unwrap(),
        ])
        .status()
        .ok()?;
    if !status.success() {
        return None;
    }
    let result = std::fs::read_to_string(&asc_path).ok()?;
    let _ = std::fs::remove_dir_all(&tmp);
    Some(result)
}

fn count_lines_starting_with(asc: &str, prefix: &str) -> usize {
    asc.lines()
        .filter(|l| l.trim().starts_with(prefix))
        .count()
}

#[test]
fn cross_pnr_simple_wire_hx1k() {
    let src = r#"module top(input a, output y); assign y = a; endmodule"#;
    let json = v2f_synth::synthesize(src, "top");

    // Pure Rust PNR
    let asc = v2f_pnr::run_pnr(&json, Device::HX1K);
    assert!(asc.contains(".device"), "missing .device");
    assert!(asc.contains("HX1K"), "wrong device");
    assert!(asc.contains(".logic_tile"), "no logic tiles");
    let sym_count = count_lines_starting_with(&asc, ".sym");
    assert!(sym_count >= 2, "should place at least 2 cells");

    // nextpnr cross-check
    if let Some(next_asc) = run_nextpnr(&json, Device::HX1K) {
        assert!(next_asc.contains(".module"));
        let io_count = count_lines_starting_with(&next_asc, ".io_tile");
        assert_eq!(io_count, 2, "2 IO ports -> 2 IO tiles");
        assert!(next_asc.contains(".synckey"), "missing synckey");
    }
}

#[test]
fn cross_pnr_small_counter_hx8k() {
    let src = r#"
module top(input clk, output reg [1:0] cnt);
always @(posedge clk) cnt <= cnt + 1;
endmodule
"#;
    let json = v2f_synth::synthesize(src, "top");

    let asc = v2f_pnr::run_pnr(&json, Device::HX8K);
    assert!(asc.contains(".device"));
    assert!(asc.contains("HX8K"));
    let sym_count = count_lines_starting_with(&asc, ".sym");
    assert!(sym_count >= 4, "should place IO + DFF + ADD cells");

    if let Some(next_asc) = run_nextpnr(&json, Device::HX8K) {
        assert!(next_asc.contains(".module"));
        assert!(next_asc.contains(".io_tile"), "no IO tiles");
        assert!(next_asc.contains(".logic_tile"), "no logic tiles");
        assert!(next_asc.contains(".synckey"));
    }
}
