use verilog2rust::{parse_verilog, gen_ruhdl, preprocess_only};
use verilog_parser::ast::*;
use std::process::{Command, Stdio};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn sim_preproc(code: &str) -> Result<String, String> {
    let expanded = preprocess_only(code);
    sim(&expanded)
}

fn sim_with_hex(code: &str, hex_content: &str) -> Result<String, String> {
    let expanded = preprocess_only(code);
    let rlib_path = format!("{}/target/debug/libverilog2rust.rlib", env!("CARGO_MANIFEST_DIR"));
    let rlib_dir = format!("{}/target/debug", env!("CARGO_MANIFEST_DIR"));
    let deps_dir = format!("{}/target/debug/deps", env!("CARGO_MANIFEST_DIR"));
    let modules = parse_verilog(&expanded);
    let rust_code = gen_ruhdl(&modules);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp = std::path::Path::new("/tmp");
    let src = tmp.join(format!("v2r_{}.rs", id));
    let out_bin = tmp.join(format!("v2r_{}", id));
    let hex_path = tmp.join("mcu0m.hex");
    fs::write(&src, &rust_code).map_err(|e| e.to_string())?;
    fs::write(&hex_path, hex_content).map_err(|e| e.to_string())?;
    let status = Command::new("rustc")
        .args(["--extern", &format!("verilog2rust={}", rlib_path),
               "-L", &rlib_dir, "-L", &deps_dir, src.to_str().unwrap(), "-o", out_bin.to_str().unwrap(), "--edition", "2021"])
        .stdout(Stdio::piped()).stderr(Stdio::piped())
        .status().map_err(|e| e.to_string())?;
    if !status.success() {
        let stderr = String::from_utf8_lossy(&{
            Command::new("rustc")
                .args(["--extern", &format!("verilog2rust={}", rlib_path),
                       "-L", &rlib_dir, "-L", &deps_dir, src.to_str().unwrap(), "-o", out_bin.to_str().unwrap(), "--edition", "2021"])
                .output().expect("rustc failed").stderr
        }).to_string();
        return Err(format!("compile failed:\n=== STDERR ===\n{}", stderr));
    }
    let output = Command::new(&out_bin).current_dir(&tmp).output().map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn sim(code: &str) -> Result<String, String> {
    let rlib_path = format!("{}/target/debug/libverilog2rust.rlib", env!("CARGO_MANIFEST_DIR"));
    let rlib_dir = format!("{}/target/debug", env!("CARGO_MANIFEST_DIR"));
    let deps_dir = format!("{}/target/debug/deps", env!("CARGO_MANIFEST_DIR"));
    let modules = parse_verilog(code);
    let rust_code = gen_ruhdl(&modules);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let src = std::path::Path::new("/tmp").join(format!("v2r_{}.rs", id));
    let out_bin = std::path::Path::new("/tmp").join(format!("v2r_{}", id));
    fs::write(&src, &rust_code).map_err(|e| e.to_string())?;
    let status = Command::new("rustc")
        .args(["--extern", &format!("verilog2rust={}", rlib_path),
               "-L", &rlib_dir, "-L", &deps_dir, src.to_str().unwrap(), "-o", out_bin.to_str().unwrap(), "--edition", "2021"])
        .stdout(Stdio::piped()).stderr(Stdio::piped())
        .status().map_err(|e| e.to_string())?;
    if !status.success() {
        let src_err = fs::read_to_string(&src).unwrap_or_default();
        let stderr = String::from_utf8_lossy(&{
            Command::new("rustc")
                .args(["--extern", &format!("verilog2rust={}", rlib_path),
                       "-L", &rlib_dir, "-L", &deps_dir, src.to_str().unwrap(), "-o", out_bin.to_str().unwrap(), "--edition", "2021"])
                .output().expect("rustc failed").stderr
        }).to_string();
        return Err(format!("compile failed:\n=== SRC ===\n{}\n=== STDERR ===\n{}", src_err, stderr));
    }
    let output = Command::new(&out_bin).output().map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

macro_rules! sim_test {
    ($name:ident, $code:expr, $check:expr) => {
        #[test]
        fn $name() {
            let out = sim($code).unwrap_or_else(|e| panic!("sim failed: {}", e));
            assert!(out.contains($check), "output should contain '{}', got: {}", $check, out);
        }
    };
}

#[test]
fn test_sim_fulladder() {
    let code = r#"module FullAdder(a, b, cin, sum, cout);
  input a, b, cin; output sum, cout;
  wire s, c1, c2;
  xor u1(s, a, b); xor u2(sum, s, cin);
  and u3(c1, a, b); and u4(c2, s, cin); or u5(cout, c1, c2);
endmodule
module tb;
  reg a, b, cin; wire sum, cout;
  FullAdder uut(a, b, cin, sum, cout);
  initial begin $display("=== FullAdder ==="); a=0; b=0; cin=0; #1 $display("a=%b b=%b cin=%b sum=%b cout=%b",a,b,cin,sum,cout); $finish; end
endmodule"#;
    let out = sim(code).unwrap();
    assert!(out.contains("FullAdder"));
}

#[test]
fn test_sim_adder4() {
    let code = r#"module FullAdder(a, b, cin, sum, cout);
  input a, b, cin; output sum, cout;
  wire s, c1, c2;
  xor u1(s, a, b); xor u2(sum, s, cin);
  and u3(c1, a, b); and u4(c2, s, cin); or u5(cout, c1, c2);
endmodule
module Adder4(a, b, cin, sum, cout);
  input [3:0] a, b; input cin; output [3:0] sum; output cout;
  wire [3:0] c;
  FullAdder fa0(a[0],b[0],cin,sum[0],c[0]);
  FullAdder fa1(a[1],b[1],c[0],sum[1],c[1]);
  FullAdder fa2(a[2],b[2],c[1],sum[2],c[2]);
  FullAdder fa3(a[3],b[3],c[2],sum[3],cout);
endmodule
module tb;
  reg [3:0] a, b; reg cin; wire [3:0] sum; wire cout;
  Adder4 uut(a, b, cin, sum, cout);
  initial begin $display("=== Adder4 ==="); a=1; b=2; cin=0; #1 $display("sum=%h cout=%b",sum,cout); $finish; end
endmodule"#;
    let out = sim(code).unwrap();
    assert!(out.contains("Adder4"));
    assert!(out.contains("sum="));
}

#[test]
fn test_sim_alu() {
    let code = r#"module ALU(a, b, op, result);
  input [7:0] a, b; input [2:0] op; output reg [7:0] result;
  always @(posedge clk) begin
    case (op) 0: result <= a + b; 1: result <= a - b; 2: result <= a & b; 3: result <= a | b; default: result <= 0; endcase
  end
endmodule
module tb;
  reg [7:0] a, b; reg [2:0] op; reg clk; wire [7:0] result;
  ALU uut(a, b, op, result);
  initial begin $display("=== ALU ===");
    clk=0; a=10; b=5; op=0;
    clk=1; #1 $display("result=%d",result);
    clk=0; op=2; clk=1; #1 $display("result=%d",result);
    $finish;
  end
endmodule"#;
    let out = sim(code).unwrap();
    assert!(out.contains("ALU"));
    assert!(out.contains("result="));
}

#[test]
fn test_sim_mux2() {
    let code = r#"module Mux2(a, b, sel, y);
  input a, b, sel; output y;
  wire not_sel, t1, t2;
  not u1(not_sel, sel); and u2(t1, a, not_sel); and u3(t2, b, sel); or u4(y, t1, t2);
endmodule
module tb;
  reg a, b, sel; wire y;
  Mux2 uut(a, b, sel, y);
  initial begin $display("=== Mux2 ==="); sel=0; a=1; b=0; #1 $display("y=%b",y); $finish; end
endmodule"#;
    let out = sim(code).unwrap();
    assert!(out.contains("Mux2"));
    assert!(out.contains("y="));
}

#[test]
fn test_sim_mux4() {
    let code = r#"module Mux4(a, b, c, d, sel, y);
  input a, b, c, d; input [1:0] sel; output y;
  wire t1, t2, t3, t4;
  and u1(t1, a, ~sel[1], ~sel[0]); and u2(t2, b, ~sel[1], sel[0]);
  and u3(t3, c, sel[1], ~sel[0]); and u4(t4, d, sel[1], sel[0]);
  or u5(y, t1, t2, t3, t4);
endmodule
module tb;
  reg a, b, c, d; reg [1:0] sel; wire y;
  Mux4 uut(a, b, c, d, sel, y);
  initial begin $display("=== Mux4 ==="); a=1; b=0; c=0; d=0; sel=0; #1 $display("y=%b",y); sel=1; #1 $display("y=%b",y); $finish; end
endmodule"#;
    let out = sim(code).unwrap();
    assert!(out.contains("Mux4"));
    assert!(out.contains("y="));
}

#[test]
fn test_sim_dff() {
    let code = r#"module DFF(d, clk, q);
  input d, clk; output reg q;
  always @(posedge clk) q <= d;
endmodule
module tb;
  reg d, clk; wire q;
  DFF uut(d, clk, q);
  initial begin $display("=== DFF ===");
    clk=0; d=0; $display("q=%b",q);
    clk=1; #1 $display("q=%b",q);
    clk=0; d=1; #1 clk=1; #1 $display("q=%b",q);
    $finish;
  end
endmodule"#;
    let out = sim(code).unwrap();
    assert!(out.contains("DFF"));
    assert!(out.contains("q="));
}

#[test]
fn test_sim_counter() {
    let code = r#"module Counter(clk, rst, count);
  input clk, rst; output [3:0] count; reg [3:0] count;
  always @(posedge clk) begin if (rst) count <= 0; else count <= count + 1; end
endmodule
module tb;
  reg clk, rst; wire [3:0] count;
  Counter uut(clk, rst, count);
  initial begin $display("=== Counter ===");
    clk=0; rst=1; $display("count=%h",count);
    clk=1; #1 rst=0;
    clk=0; #1 $display("count=%h",count);
    clk=1; #1 $display("count=%h",count);
    $finish;
  end
endmodule"#;
    let out = sim(code).unwrap();
    assert!(out.contains("Counter"));
    assert!(out.contains("count="));
}

#[test]
fn test_sim_decoder() {
    let code = r#"module Decoder2x4(enable, idx, out);
  input enable; input [1:0] idx; output [3:0] out;
  wire [1:0] not_idx;
  not u0(not_idx[0], idx[0]); not u1(not_idx[1], idx[1]);
  and u2(out[0], enable, not_idx[1], not_idx[0]);
  and u3(out[1], enable, not_idx[1], idx[0]);
  and u4(out[2], enable, idx[1], not_idx[0]);
  and u5(out[3], enable, idx[1], idx[0]);
endmodule
module tb;
  reg enable; reg [1:0] idx; wire [3:0] out;
  Decoder2x4 uut(enable, idx, out);
  initial begin $display("=== Decoder ==="); enable=1; idx=0; #1 $display("out=%b%b%b%b",out[3],out[2],out[1],out[0]); $finish; end
endmodule"#;
    let out = sim(code).unwrap();
    assert!(out.contains("Decoder"));
    assert!(out.contains("out="));
}

#[test]
fn test_sim_adder8() {
    let code = r#"module FullAdder(a, b, cin, sum, cout);
  input a, b, cin; output sum, cout;
  wire s, c1, c2;
  xor u1(s, a, b); xor u2(sum, s, cin);
  and u3(c1, a, b); and u4(c2, s, cin); or u5(cout, c1, c2);
endmodule
module Adder4(a, b, cin, sum, cout);
  input [3:0] a, b; input cin; output [3:0] sum; output cout;
  wire [3:0] c;
  FullAdder fa0(a[0],b[0],cin,sum[0],c[0]);
  FullAdder fa1(a[1],b[1],c[0],sum[1],c[1]);
  FullAdder fa2(a[2],b[2],c[1],sum[2],c[2]);
  FullAdder fa3(a[3],b[3],c[2],sum[3],cout);
endmodule
module Adder8(a, b, cin, sum, cout);
  input [7:0] a, b; input cin; output [7:0] sum; output cout;
  wire c4;
  Adder4 low(a[3:0],b[3:0],cin,sum[3:0],c4);
  Adder4 high(a[7:4],b[7:4],c4,sum[7:4],cout);
endmodule
module tb;
  reg [7:0] a, b; reg cin; wire [7:0] sum; wire cout;
  Adder8 uut(a, b, cin, sum, cout);
  initial begin $display("=== Adder8 ==="); a=8'h55; b=8'h2A; cin=0; #1 $display("sum=%h",sum); $finish; end
endmodule"#;
    let out = sim(code).unwrap();
    assert!(out.contains("Adder8"));
    assert!(out.contains("sum="));
}

#[test]
fn test_sim_register() {
    let code = r#"module Register(clk, d, q);
  input clk, d; output reg q;
  always @(posedge clk) q <= d;
endmodule
module tb;
  reg clk, d; wire q;
  Register uut(clk, d, q);
  initial begin $display("=== Register ==="); clk=0; d=0; $display("q=%b",q); clk=1; #1 $display("q=%b",q); clk=0; d=1; #1 clk=1; #1 $display("q=%b",q); $finish; end
endmodule"#;
    let out = sim(code).unwrap();
    assert!(out.contains("Register"));
    assert!(out.contains("q="));
}

#[test]
fn test_sim_fsm() {
    let code = r#"module FSM(clk, rst, inp, out);
  input clk, rst, inp; output reg [1:0] out; reg [1:0] state;
  always @(posedge clk) begin
    if (rst) state <= 0;
    else if (state == 0) begin if (inp) state <= 1; else state <= 0; end
    else if (state == 1) begin if (inp) state <= 2; else state <= 0; end
    else state <= 0;
  end
  always @(*) begin if (state == 0) out = 1; else if (state == 1) out = 2; else out = 3; end
endmodule
module tb;
  reg clk, rst, inp; wire [1:0] out;
  FSM uut(clk, rst, inp, out);
  initial begin $display("=== FSM ==="); clk=0; rst=1; inp=0; $display("out=%b",out); clk=1; #1 rst=0; $display("out=%b",out); inp=1; clk=0; #1 clk=1; $display("out=%b",out); $finish; end
endmodule"#;
    let out = sim(code).unwrap();
    assert!(out.contains("FSM"));
    assert!(out.contains("out="));
}

#[test]
fn test_tokenize_simple() {
    let input = "module foo(a,b); input a,b; wire s; and g(s,a,b); endmodule";
    let modules = parse_verilog(input);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].name, "foo");
    assert_eq!(modules[0].ports.len(), 2);
}

#[test]
fn test_tokenize_ports() {
    let input = "\
module test(data, result, clk);
    input [3:0] data;
    output reg [7:0] result;
    input clk;
    wire [3:0] tmp;
    reg [7:0] accum;
endmodule";
    let modules = parse_verilog(input);
    assert_eq!(modules.len(), 1);
    let m = &modules[0];
    assert_eq!(m.name, "test");
    assert_eq!(m.ports.len(), 3);

    let data_port = m.ports.iter().find(|p| p.name == "data").unwrap();
    assert_eq!(data_port.direction, PortDir::Input);
    assert_eq!(data_port.width.as_ref().unwrap().msb, 3);
    assert_eq!(data_port.width.as_ref().unwrap().lsb, 0);

    let clk_port = m.ports.iter().find(|p| p.name == "clk").unwrap();
    assert_eq!(clk_port.direction, PortDir::Input);
    assert!(clk_port.width.is_none());
}

#[test]
fn test_gate_instantiations() {
    let input = "\
module FullAdder(a, b, cin, sum, cout);
    input a, b, cin;
    output sum, cout;
    wire s, c1, c2;

    xor u1(s, a, b);
    xor u2(sum, s, cin);
    and u3(c1, a, b);
    and u4(c2, s, cin);
    or u5(cout, c1, c2);
endmodule";
    let modules = parse_verilog(input);
    assert_eq!(modules.len(), 1);
    let m = &modules[0];
    let gates: Vec<&GateInst> = m.items.iter().filter_map(|item| {
        if let ModuleItem::GateInst(g) = item { Some(g) } else { None }
    }).collect();
    assert_eq!(gates.len(), 5);

    let xor1 = &gates[0];
    assert_eq!(xor1.gate_type, "xor");
    assert_eq!(xor1.instance_name, "u1");
}

#[test]
fn test_module_instantiation() {
    let input = "\
module Adder4(a, b, cin, sum, cout);
    input [3:0] a, b;
    input cin;
    output [3:0] sum;
    output cout;
    wire [3:0] c;

    FullAdder fa0(.a(a[0]), .b(b[0]), .cin(cin), .sum(sum[0]), .cout(c[0]));
    FullAdder fa1(.a(a[1]), .b(b[1]), .cin(c[0]), .sum(sum[1]), .cout(c[1]));
endmodule";
    let modules = parse_verilog(input);
    assert_eq!(modules.len(), 1);
    let m = &modules[0];
    let insts: Vec<&ModuleInst> = m.items.iter().filter_map(|item| {
        if let ModuleItem::ModuleInst(s) = item { Some(s) } else { None }
    }).collect();
    assert_eq!(insts.len(), 2);
    assert_eq!(insts[0].module_name, "FullAdder");
    assert_eq!(insts[0].instance_name, "fa0");
}

#[test]
fn test_assign() {
    let input = "\
module test(a, b, y);
    input a, b;
    output y;
    assign y = a & b;
endmodule";
    let modules = parse_verilog(input);
    assert_eq!(modules.len(), 1);
    let m = &modules[0];
    let assign_count = m.items.iter().filter(|item| {
        matches!(item, ModuleItem::Assign { .. })
    }).count();
    assert_eq!(assign_count, 1);
}

#[test]
fn test_always_block() {
    let input = "\
module Counter(clk, rst, q);
    input clk, rst;
    output reg [7:0] q;

    always @(posedge clk or posedge rst) begin
        if (rst)
            q <= 8'b00000000;
        else
            q <= q + 1;
    end
endmodule";
    let modules = parse_verilog(input);
    assert_eq!(modules.len(), 1);
    let m = &modules[0];
    let always: Vec<&AlwaysBlock> = m.items.iter().filter_map(|item| {
        if let ModuleItem::Always(a) = item { Some(a) } else { None }
    }).collect();
    assert_eq!(always.len(), 1);
    assert_eq!(always[0].stmts.len(), 1);
}

#[test]
fn test_always_star() {
    let input = "\
module ALU(a, b, op, result);
    input [3:0] a, b;
    input [1:0] op;
    output reg [3:0] result;

    always @(*) begin
        case (op)
            2'b00: result = a + b;
            2'b01: result = a - b;
            2'b10: result = a & b;
            2'b11: result = a | b;
        endcase
    end
endmodule";
    let modules = parse_verilog(input);
    assert_eq!(modules.len(), 1);
    let m = &modules[0];
    let always: Vec<&AlwaysBlock> = m.items.iter().filter_map(|item| {
        if let ModuleItem::Always(a) = item { Some(a) } else { None }
    }).collect();
    assert_eq!(always.len(), 1);
    assert!(always[0].sensitivity.contains(&Sensitivity::All));
}

#[test]
fn test_fulladder_gen_output() {
    let input = "\
module FullAdder(a, b, cin, sum, cout);
    input a, b, cin;
    output sum, cout;
    wire s, c1, c2;

    xor u1(s, a, b);
    xor u2(sum, s, cin);
    and u3(c1, a, b);
    and u4(c2, s, cin);
    or u5(cout, c1, c2);
endmodule";
    let modules = parse_verilog(input);
    let code = gen_ruhdl(&modules);
    assert!(code.contains("pub struct FullAdder"));
    assert!(code.contains("Xor::new(vec![a.clone(), b.clone()], s.clone())"));
    assert!(code.contains("Xor::new(vec![s.clone(), cin.clone()], sum.clone())"));
    assert!(code.contains("And::new(vec![a.clone(), b.clone()], c1.clone())"));
    assert!(code.contains("Or::new(vec![c1.clone(), c2.clone()], cout.clone())"));
}

#[test]
fn test_mux2_gen_output() {
    let input = "\
module Mux2(a, b, sel, y);
    input a, b, sel;
    output y;
    wire not_sel, t1, t2;

    not u1(not_sel, sel);
    and u2(t1, a, not_sel);
    and u3(t2, b, sel);
    or u4(y, t1, t2);
endmodule";
    let modules = parse_verilog(input);
    let code = gen_ruhdl(&modules);
    assert!(code.contains("Not::new(vec![sel.clone()], not_sel.clone())"));
    assert!(code.contains("And::new(vec![a.clone(), not_sel.clone()], t1.clone())"));
    assert!(code.contains("Or::new(vec![t1.clone(), t2.clone()], y.clone())"));
}

#[test]
fn test_adder4_gen_with_submodules() {
    let input = "\
module FullAdder(a, b, cin, sum, cout);
    input a, b, cin;
    output sum, cout;
    wire s, c1, c2;
    xor u1(s, a, b);
    xor u2(sum, s, cin);
    and u3(c1, a, b);
    and u4(c2, s, cin);
    or u5(cout, c1, c2);
endmodule

module Adder4(a, b, cin, sum, cout);
    input [3:0] a, b;
    input cin;
    output [3:0] sum;
    output cout;
    wire [3:0] c;

    FullAdder fa0(.a(a[0]), .b(b[0]), .cin(cin), .sum(sum[0]), .cout(c[0]));
    FullAdder fa1(.a(a[1]), .b(b[1]), .cin(c[0]), .sum(sum[1]), .cout(c[1]));
endmodule";
    let modules = parse_verilog(input);
    let code = gen_ruhdl(&modules);
    assert!(code.contains("pub struct Adder4"));
    assert!(code.contains("fa0: FullAdder"));
    assert!(code.contains("fa0.eval"));
}

#[test]
fn test_numbers() {
    let input = "\
module test(q);
    output reg [7:0] q;
    always @(*) begin
        q = 8'hAB;
    end
endmodule";
    let modules = parse_verilog(input);
    assert_eq!(modules.len(), 1);
    let m = &modules[0];
    let always: Vec<&AlwaysBlock> = m.items.iter().filter_map(|item| {
        if let ModuleItem::Always(a) = item { Some(a) } else { None }
    }).collect();
    assert_eq!(always.len(), 1);
}

#[test]
fn test_concat_and_select() {
    let input = "\
module test(a, b, y);
    input [7:0] a;
    input [7:0] b;
    output [15:0] y;
    assign y = {a, b};
endmodule";
    let modules = parse_verilog(input);
    assert_eq!(modules.len(), 1);
}

#[test]
fn test_counter_gen_contains_eval_logic() {
    let input = "\
module Counter(clk, rst, en, q);
    input clk, rst, en;
    output reg [7:0] q;

    always @(posedge clk) begin
        if (rst)
            q <= 8'b00000000;
        else if (en)
            q <= q + 1;
    end
endmodule";
    let modules = parse_verilog(input);
    let code = gen_ruhdl(&modules);
    assert!(code.contains("pub struct Counter"));
    assert!(code.contains("fn eval"));
    assert!(code.contains("u16_to_bus"));
}

#[test]
fn test_gen_output_is_valid_rust_syntax() {
    let input = "\
module FullAdder(a, b, cin, sum, cout);
    input a, b, cin;
    output sum, cout;
    wire s, c1, c2;
    xor u1(s, a, b);
    xor u2(sum, s, cin);
    and u3(c1, a, b);
    and u4(c2, s, cin);
    or u5(cout, c1, c2);
endmodule";
    let modules = parse_verilog(input);
    let code = gen_ruhdl(&modules);
    // Basic sanity: check that braces are balanced
    let opens = code.matches('{').count();
    let closes = code.matches('}').count();
    assert_eq!(opens, closes, "Braces in generated code should be balanced");
}

#[test]
fn test_verilog_comments() {
    let input = "\
// This is a comment
module test(a, b);
    /* multi-line
       comment */
    input a, b;
    wire s;
    and g(s, a, b);
endmodule";
    let modules = parse_verilog(input);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].name, "test");
}

#[test]
fn test_bitwise_operators() {
    let input = "\
module test(a, b, y);
    input [3:0] a, b;
    output [3:0] y;
    assign y = a & b;
endmodule";
    let modules = parse_verilog(input);
    assert_eq!(modules.len(), 1);
}

#[test]
fn test_multiple_modules() {
    let input = "\
module A(x, y);
    input x;
    output y;
    not g(y, x);
endmodule

module B(x, y);
    input x;
    output y;
    not g(y, x);
endmodule";
    let modules = parse_verilog(input);
    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].name, "A");
    assert_eq!(modules[1].name, "B");
}

// ----- new feature tests: initial, $display, $finish, #delay, no-port modules -----

#[test]
fn test_module_no_ports() {
    let input = "\
module test;
    reg a;
    wire b;
endmodule";
    let modules = parse_verilog(input);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].name, "test");
    assert!(modules[0].ports.is_empty());
}

#[test]
fn test_initial_block() {
    let input = "\
module test;
    reg a;
    initial begin
        a = 1'b1;
    end
endmodule";
    let modules = parse_verilog(input);
    assert_eq!(modules.len(), 1);
    let m = &modules[0];
    let initial_count = m.items.iter().filter(|item| {
        matches!(item, ModuleItem::Initial(_))
    }).count();
    assert_eq!(initial_count, 1);
}

#[test]
fn test_syscall_display() {
    let input = "\
module test;
    reg [3:0] a;
    initial begin
        $display(\"hello\");
        $display(\"val = %d\", a);
        $finish;
    end
endmodule";
    let modules = parse_verilog(input);
    assert_eq!(modules.len(), 1);
}

#[test]
fn test_delay_stmt() {
    let input = "\
module test;
    reg a;
    initial begin
        #10;
        a = 1'b1;
        #5 a = 1'b0;
    end
endmodule";
    let modules = parse_verilog(input);
    assert_eq!(modules.len(), 1);
    let m = &modules[0];
    let init_idx = m.items.iter().position(|item| matches!(item, ModuleItem::Initial(_))).unwrap();
    if let ModuleItem::Initial(stmts) = &m.items[init_idx] {
        assert_eq!(stmts.len(), 3);
    } else {
        panic!("expected initial block");
    }
}

#[test]
fn test_tb_gen_has_main() {
    let input = "\
module FullAdder(a, b, cin, sum, cout);
    input a, b, cin;
    output sum, cout;
    wire s, c1, c2;
    xor u1(s, a, b);
    xor u2(sum, s, cin);
    and u3(c1, a, b);
    and u4(c2, s, cin);
    or u5(cout, c1, c2);
endmodule

module tb;
    reg a, b, cin;
    wire sum, cout;
    FullAdder dut(a, b, cin, sum, cout);
    initial begin
        $display(\"test\");
        $finish;
    end
endmodule";
    let modules = parse_verilog(input);
    let code = gen_ruhdl(&modules);
    assert!(code.contains("fn main()"));
    assert!(code.contains("tb.run()"));
    assert!(code.contains("println!"));
    assert!(code.contains("return;"));
}

#[test]
fn test_gen_mcu0m() {
    let code = include_str!("../verilog/mcu0m.v");
    let modules = parse_verilog(code);
    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].name, "cpu");
    assert_eq!(modules[0].ports.len(), 1);
    assert_eq!(modules[0].ports[0].name, "clock");
    assert_eq!(modules[1].name, "main");
    let rust_code = gen_ruhdl(&modules);
    let opens = rust_code.matches('{').count();
    let closes = rust_code.matches('}').count();
    assert_eq!(opens, closes, "Braces in generated code should be balanced");
    assert!(rust_code.contains("pub struct Cpu"));
    assert!(rust_code.contains("pub struct Main"));
}

#[test]
fn test_sim_mcu0m() {
    let code = include_str!("../verilog/mcu0m.v");
    let out = sim(code).unwrap_or_else(|e| panic!("sim failed: {}", e));
    assert!(out.contains("Memory dump:"), "should have memory dump, got: {:?}", out);
    assert!(out.contains("SW=8000"), "should have CMP less-than flag set, full: {:?}", out);
    assert!(out.contains("=  55"), "should sum 1..10 to 55, full: {:?}", out);
    assert!(out.contains("SW=4000"), "should have JEQ equal flag at exit, full: {:?}", out);
}

#[test]
fn test_sim_mcu0m1() {
    let code = include_str!("../verilog/mcu0m1.v");
    let out = sim_preproc(code).unwrap_or_else(|e| panic!("sim failed: {}", e));
    assert!(out.contains("SW=8000"), "should have CMP less-than flag set, full: {:?}", out);
    assert!(out.contains("A=55"), "should sum 1..10 to 55, full: {:?}", out);
    assert!(out.contains("SW=4000"), "should have JEQ equal flag at exit, full: {:?}", out);
}

#[test]
fn test_sim_mcu0m_from_mcu0_dir() {
    let code = include_str!("../verilog/mcu0/mcu0m.v");
    let hex = include_str!("../verilog/mcu0/mcu0m.hex");
    let out = sim_with_hex(code, hex).unwrap_or_else(|e| panic!("sim failed: {}", e));
    assert!(out.contains("SW=8000"), "should have CMP less-than flag set, full: {:?}", out);
    assert!(out.contains("A=55"), "should sum 1..10 to 55, full: {:?}", out);
    assert!(out.contains("SW=4000"), "should have JEQ equal flag at exit, full: {:?}", out);
}

#[test]
fn test_initial_gen_run_method() {
    let input = "\
module tb;
    reg a;
    initial begin
        a = 1'b1;
    end
endmodule";
    let modules = parse_verilog(input);
    let code = gen_ruhdl(&modules);
    assert!(code.contains("pub fn run"));
}
