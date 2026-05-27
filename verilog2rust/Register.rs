use verilog2rust::rhdl::prelude::*;

#[derive(Debug, Clone)]
pub struct Register {
    pub d: Vec<WireRef>,
    pub q: Vec<WireRef>,
    pub clk: WireRef,
    pub load: WireRef,
}

impl Register {
    pub fn new(
        d: Vec<WireRef>,
        q: Vec<WireRef>,
        clk: WireRef,
        load: WireRef,
    ) -> Self {
        Register {
            d: d.clone(),
            q: q.clone(),
            clk: clk.clone(),
            load: load.clone(),
        }
    }

    pub fn eval(&mut self) {
    if get(&self.load) != Level::L {
        u16_to_bus(&self.q, (bus_to_u16(&self.d) as u64 & 15u64) as u16);
    }
    }
}

