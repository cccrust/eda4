use verilog2rust::{parse_verilog, gen_ruhdl};
use verilog2rust::verilog::ast::*;

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
    assert!(code.contains("Xor::new(a.clone(), b.clone(), s.clone())"));
    assert!(code.contains("Xor::new(s.clone(), cin.clone(), sum.clone())"));
    assert!(code.contains("And::new(a.clone(), b.clone(), c1.clone())"));
    assert!(code.contains("Or::new(c1.clone(), c2.clone(), cout.clone())"));
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
    assert!(code.contains("Not::new(sel.clone(), not_sel.clone())"));
    assert!(code.contains("And::new(a.clone(), not_sel.clone(), t1.clone())"));
    assert!(code.contains("Or::new(t1.clone(), t2.clone(), y.clone())"));
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
