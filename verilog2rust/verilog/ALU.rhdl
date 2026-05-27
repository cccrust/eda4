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
}

