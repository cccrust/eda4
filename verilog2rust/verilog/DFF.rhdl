use verilog2rust::rhdl::prelude::*;

#[derive(Debug, Clone)]
pub struct DFF {
    pub d: WireRef,
    pub clk: WireRef,
    pub q: WireRef,
}

impl DFF {
    pub fn new(
        d: WireRef,
        clk: WireRef,
        q: WireRef,
    ) -> Self {
        DFF {
            d: d.clone(),
            clk: clk.clone(),
            q: q.clone(),
        }
    }

    pub fn eval(&mut self) {
    if get(&self.q) != get(&self.d) { set(&self.q, get(&self.d)); }
    }
}

