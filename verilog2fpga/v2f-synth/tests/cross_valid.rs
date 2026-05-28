use std::process::Command;

fn run_yosys_synth(verilog: &str, top: &str) -> Option<String> {
    if !Command::new("yosys").arg("--version").output().is_ok() {
        return None;
    }
    let tmp = std::env::temp_dir().join(format!("v2f_xv_synth_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).ok()?;
    let src_path = tmp.join("input.v");
    let out_path = tmp.join("output.json");
    std::fs::write(&src_path, verilog).ok()?;
    let status = Command::new("yosys")
        .args(["-q", "-p", &format!(
            "read_verilog {}; synth_ice40 -top {} -json {}",
            src_path.display(),
            top,
            out_path.display()
        )])
        .status()
        .ok()?;
    if !status.success() {
        return None;
    }
    let result = std::fs::read_to_string(&out_path).ok()?;
    let _ = std::fs::remove_dir_all(&tmp);
    Some(result)
}

fn parse_module(json_str: &str, mod_name: &str) -> serde_json::Value {
    let v: serde_json::Value = serde_json::from_str(json_str).unwrap();
    v["modules"][mod_name].clone()
}

fn blinky_src() -> &'static str {
    r#"
module blinky(clk, led);
input clk;
output led;
reg [25:0] counter;
always @(posedge clk) counter <= counter + 1;
assign led = counter[25];
endmodule
"#
}

fn adder_src() -> &'static str {
    r#"
module adder(a, b, sum, carry);
input [3:0] a;
input [3:0] b;
output [3:0] sum;
output carry;
wire [4:0] result;
assign result = a + b;
assign sum = result[3:0];
assign carry = result[4];
endmodule
"#
}

#[test]
fn cross_synth_blinky_ports_structure() {
    let pure = parse_module(&v2f_synth::synthesize(blinky_src(), "blinky"), "blinky");
    let ports = pure["ports"].as_object().unwrap();
    assert_eq!(ports.len(), 2);
    assert_eq!(ports["clk"]["direction"], "input");
    assert_eq!(ports["clk"]["bits"].as_array().unwrap().len(), 1);
    assert_eq!(ports["led"]["direction"], "output");
    assert_eq!(ports["led"]["bits"].as_array().unwrap().len(), 1);

    if let Some(yosys_json) = run_yosys_synth(blinky_src(), "blinky") {
        let yosys = parse_module(&yosys_json, "blinky");
        let y_ports = yosys["ports"].as_object().unwrap();
        assert_eq!(ports.len(), y_ports.len());
        for name in ports.keys() {
            let p = &ports[name];
            let y = &y_ports[name.as_str()];
            assert_eq!(p["direction"], y["direction"], "port {name} dir");
            assert_eq!(
                p["bits"].as_array().unwrap().len(),
                y["bits"].as_array().unwrap().len(),
                "port {name} width"
            );
        }
    }
}

#[test]
fn cross_synth_blinky_dff_count() {
    let pure = parse_module(&v2f_synth::synthesize(blinky_src(), "blinky"), "blinky");
    let cells = pure["cells"].as_object().unwrap();
    let dff_count = cells
        .values()
        .filter(|c| c["type"] == "$_DFF_P_")
        .count();
    assert_eq!(dff_count, 26, "26-bit counter needs 26 DFF cells");
}

#[test]
fn cross_synth_adder_ports_structure() {
    let pure = parse_module(&v2f_synth::synthesize(adder_src(), "adder"), "adder");
    let ports = pure["ports"].as_object().unwrap();
    assert_eq!(ports.len(), 4);
    assert_eq!(ports["a"]["bits"].as_array().unwrap().len(), 4);
    assert_eq!(ports["b"]["bits"].as_array().unwrap().len(), 4);
    assert_eq!(ports["sum"]["bits"].as_array().unwrap().len(), 4);
    assert_eq!(ports["carry"]["bits"].as_array().unwrap().len(), 1);

    if let Some(yosys_json) = run_yosys_synth(adder_src(), "adder") {
        let yosys = parse_module(&yosys_json, "adder");
        let y_ports = yosys["ports"].as_object().unwrap();
        assert_eq!(ports.len(), y_ports.len());
        for name in ports.keys() {
            assert_eq!(
                ports[name]["direction"],
                y_ports[name.as_str()]["direction"],
                "port {name} dir"
            );
            assert_eq!(
                ports[name]["bits"].as_array().unwrap().len(),
                y_ports[name.as_str()]["bits"].as_array().unwrap().len(),
                "port {name} width"
            );
        }
    }
}

#[test]
fn cross_synth_adder_no_dff() {
    let pure = parse_module(&v2f_synth::synthesize(adder_src(), "adder"), "adder");
    let cells = pure["cells"].as_object().unwrap();
    let dff_count = cells
        .values()
        .filter(|c| c["type"] == "$_DFF_P_")
        .count();
    assert_eq!(dff_count, 0, "adder should have no sequential cells");
}
