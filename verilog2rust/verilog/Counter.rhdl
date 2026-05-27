use verilog2rust::rhdl::prelude::*;

#[derive(Debug, Clone)]
pub struct Counter {
    pub clk: WireRef,
    pub rst: WireRef,
    pub en: WireRef,
    pub q: Vec<WireRef>,
}

impl Counter {
    pub fn new(
        clk: WireRef,
        rst: WireRef,
        en: WireRef,
        q: Vec<WireRef>,
    ) -> Self {
        Counter {
            clk: clk.clone(),
            rst: rst.clone(),
            en: en.clone(),
            q: q.clone(),
        }
    }

    pub fn eval(&mut self) {
    if get(&self.rst) != Level::L {
        u16_to_bus(&self.q, (0 & 255u64) as u16);
    } else {
        if get(&self.en) != Level::L {
            u16_to_bus(&self.q, ((bus_to_u16(&self.q) as u64 + 1) & 255u64 & 255u64) as u16);
        }
    }
    }
}

