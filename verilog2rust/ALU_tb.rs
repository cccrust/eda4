use verilog2rust::rhdl::prelude::*;

#[derive(Debug, Clone)]
pub struct ALU {
    pub a: Vec<WireRef>,
    pub b: Vec<WireRef>,
    pub op: Vec<WireRef>,
    pub result: Vec<WireRef>,
    pub zero: WireRef,
}

impl ALU {
    pub fn new(
        a: Vec<WireRef>,
        b: Vec<WireRef>,
        op: Vec<WireRef>,
        result: Vec<WireRef>,
        zero: WireRef,
    ) -> Self {
        ALU {
            a: a.clone(),
            b: b.clone(),
            op: op.clone(),
            result: result.clone(),
            zero: zero.clone(),
        }
    }

    pub fn eval(&mut self) {
    let __case_val = bus_to_u16(&self.op) as u64;
    if __case_val == 0 {
        u16_to_bus(&self.result, ((bus_to_u16(&self.a) as u64 + bus_to_u16(&self.b) as u64) & 15u64 & 15u64) as u16);
    }
    if __case_val == 1 {
        u16_to_bus(&self.result, (((bus_to_u16(&self.a) as u64).wrapping_sub(bus_to_u16(&self.b) as u64)) & 15u64 & 15u64) as u16);
    }
    if __case_val == 2 {
        u16_to_bus(&self.result, ((bus_to_u16(&self.a) as u64 & bus_to_u16(&self.b) as u64) & 15u64) as u16);
    }
    if __case_val == 3 {
        u16_to_bus(&self.result, ((bus_to_u16(&self.a) as u64 | bus_to_u16(&self.b) as u64) & 15u64) as u16);
    }
    if get(&self.zero) != if (bus_to_u16(&self.result) as u64) == (0) { Level::H } else { Level::L } { set(&self.zero, if (bus_to_u16(&self.result) as u64) == (0) { Level::H } else { Level::L }); }
    }
    pub fn run(&mut self) {
    }
}

#[derive(Debug, Clone)]
pub struct ALUTb {
    result: Vec<WireRef>,
    zero: WireRef,
    a: Vec<WireRef>,
    b: Vec<WireRef>,
    op: Vec<WireRef>,
    dut: ALU,
}

impl ALUTb {
    pub fn new(
    ) -> Self {
        let result = bus("result", 4);
        let zero = wire("zero");
        let a = bus("a", 4);
        let b = bus("b", 4);
        let op = bus("op", 2);
        ALUTb {
            result: result.clone(),
            zero: zero.clone(),
            a: a.clone(),
            b: b.clone(),
            op: op.clone(),
            dut: ALU::new(a.clone(), b.clone(), op.clone(), result.clone(), zero.clone()),
        }
    }

    pub fn eval(&mut self) {
        self.dut.eval();
    }
    pub fn run(&mut self) {
        self.dut.run();
        println!("=== ALU Testbench ===");
        println!(" op |   a    b  | result  zero");
        println!("----+----------+---------");
        u16_to_bus(&self.a, (3 & 15u64) as u16);
        u16_to_bus(&self.b, (5 & 15u64) as u16);
        u16_to_bus(&self.op, (0 & 3u64) as u16);
        self.eval();
        println!(" ADD | {}d  + {}d |    {}d    {}", bus_to_u16(&self.a) as u64, bus_to_u16(&self.b) as u64, bus_to_u16(&self.result) as u64, get(&self.zero) as u64);
        u16_to_bus(&self.a, (9 & 15u64) as u16);
        u16_to_bus(&self.b, (7 & 15u64) as u16);
        u16_to_bus(&self.op, (0 & 3u64) as u16);
        self.eval();
        println!(" ADD | {}d  + {}d |    {}d    {}", bus_to_u16(&self.a) as u64, bus_to_u16(&self.b) as u64, bus_to_u16(&self.result) as u64, get(&self.zero) as u64);
        u16_to_bus(&self.a, (0 & 15u64) as u16);
        u16_to_bus(&self.b, (0 & 15u64) as u16);
        u16_to_bus(&self.op, (0 & 3u64) as u16);
        self.eval();
        println!(" ADD | {}d  + {}d |    {}d    {}", bus_to_u16(&self.a) as u64, bus_to_u16(&self.b) as u64, bus_to_u16(&self.result) as u64, get(&self.zero) as u64);
        u16_to_bus(&self.a, (8 & 15u64) as u16);
        u16_to_bus(&self.b, (3 & 15u64) as u16);
        u16_to_bus(&self.op, (1 & 3u64) as u16);
        self.eval();
        println!(" SUB | {}d  - {}d |    {}d    {}", bus_to_u16(&self.a) as u64, bus_to_u16(&self.b) as u64, bus_to_u16(&self.result) as u64, get(&self.zero) as u64);
        u16_to_bus(&self.a, (3 & 15u64) as u16);
        u16_to_bus(&self.b, (8 & 15u64) as u16);
        u16_to_bus(&self.op, (1 & 3u64) as u16);
        self.eval();
        println!(" SUB | {}d  - {}d |    {}d    {}", bus_to_u16(&self.a) as u64, bus_to_u16(&self.b) as u64, bus_to_u16(&self.result) as u64, get(&self.zero) as u64);
        u16_to_bus(&self.a, (12 & 15u64) as u16);
        u16_to_bus(&self.b, (10 & 15u64) as u16);
        u16_to_bus(&self.op, (2 & 3u64) as u16);
        self.eval();
        println!(" AND | {}d  & {}d |    {}d    {}", bus_to_u16(&self.a) as u64, bus_to_u16(&self.b) as u64, bus_to_u16(&self.result) as u64, get(&self.zero) as u64);
        u16_to_bus(&self.a, (12 & 15u64) as u16);
        u16_to_bus(&self.b, (3 & 15u64) as u16);
        u16_to_bus(&self.op, (3 & 3u64) as u16);
        self.eval();
        println!(" OR  | {}d  | {}d |    {}d    {}", bus_to_u16(&self.a) as u64, bus_to_u16(&self.b) as u64, bus_to_u16(&self.result) as u64, get(&self.zero) as u64);
        u16_to_bus(&self.a, (0 & 15u64) as u16);
        u16_to_bus(&self.b, (0 & 15u64) as u16);
        u16_to_bus(&self.op, (3 & 3u64) as u16);
        self.eval();
        println!(" OR  | {}d  | {}d |    {}d    {}", bus_to_u16(&self.a) as u64, bus_to_u16(&self.b) as u64, bus_to_u16(&self.result) as u64, get(&self.zero) as u64);
        return;
    }
}

fn main() {
    let mut tb = ALUTb::new();
    tb.run();
}
