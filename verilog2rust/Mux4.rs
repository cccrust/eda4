use verilog2rust::rhdl::prelude::*;

#[derive(Debug, Clone)]
pub struct Mux4 {
    pub i0: WireRef,
    pub i1: WireRef,
    pub i2: WireRef,
    pub i3: WireRef,
    pub sel: Vec<WireRef>,
    pub y: WireRef,
}

impl Mux4 {
    pub fn new(
        i0: WireRef,
        i1: WireRef,
        i2: WireRef,
        i3: WireRef,
        sel: Vec<WireRef>,
        y: WireRef,
    ) -> Self {
        Mux4 {
            i0: i0.clone(),
            i1: i1.clone(),
            i2: i2.clone(),
            i3: i3.clone(),
            sel: sel.clone(),
            y: y.clone(),
        }
    }

    pub fn eval(&mut self) {
    let __case_val = bus_to_u16(&self.sel) as u64;
    if __case_val == 0 {
        if get(&self.y) != get(&self.i0) { set(&self.y, get(&self.i0)); }
    }
    if __case_val == 1 {
        if get(&self.y) != get(&self.i1) { set(&self.y, get(&self.i1)); }
    }
    if __case_val == 2 {
        if get(&self.y) != get(&self.i2) { set(&self.y, get(&self.i2)); }
    }
    if __case_val == 3 {
        if get(&self.y) != get(&self.i3) { set(&self.y, get(&self.i3)); }
    }
    }
}

