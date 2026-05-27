use crate::parser::Parser;
use crate::elab::elaborate;
use crate::techmap::techmap_to_json;

pub fn synthesize(src: &str, _top: &str) -> String {
    let mut parser = Parser::new(src);
    let module = parser.parse_module();
    let netlist = elaborate(&module);
    let json_modules = techmap_to_json(&netlist);

    let output = serde_json::to_string_pretty(&serde_json::json!({
        "creator": "v2f-synth v0.3",
        "modules": json_modules,
    })).unwrap();

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synth_simple_wire() {
        let src = "module top(input a, output y); assign y = a; endmodule";
        let json = synthesize(src, "top");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["creator"], "v2f-synth v0.3");
        assert_eq!(parsed["modules"]["top"]["ports"]["a"]["direction"], "input");
        assert_eq!(parsed["modules"]["top"]["ports"]["y"]["direction"], "output");
    }

    #[test]
    fn test_synth_blinky() {
        let src = r#"
module blinky(input clk, output reg led);
reg [25:0] counter;
always @(posedge clk) begin
    counter <= counter + 1;
end
assign led = counter[25];
endmodule
"#;
        let json = synthesize(src, "blinky");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["creator"], "v2f-synth v0.3");
        let cells = parsed["modules"]["blinky"]["cells"].as_object().unwrap();
        let has_dff = cells.values().any(|c| c["type"] == "$_DFF_P_");
        assert!(has_dff, "should have at least one DFF cell");
    }

    #[test]
    fn test_synth_adder() {
        let src = r#"
module adder(input [3:0] a, input [3:0] b, output [3:0] sum, output carry);
wire [4:0] result;
assign result = a + b;
assign sum = result[3:0];
assign carry = result[4];
endmodule
"#;
        let json = synthesize(src, "adder");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["creator"], "v2f-synth v0.3");
        assert_eq!(parsed["modules"]["adder"]["ports"]["sum"]["direction"], "output");
        assert_eq!(parsed["modules"]["adder"]["ports"]["carry"]["direction"], "output");
    }
}
