use verilog2rust::rhdl::prelude::*;

#[derive(Debug, Clone)]
pub struct HalfAdder {
    pub a: WireRef,
    pub b: WireRef,
    pub sum: WireRef,
    pub carry: WireRef,
    u1: Xor,
    u2: And,
}

impl HalfAdder {
    pub fn new(
        a: WireRef,
        b: WireRef,
        sum: WireRef,
        carry: WireRef,
    ) -> Self {
        HalfAdder {
            a: a.clone(),
            b: b.clone(),
            sum: sum.clone(),
            carry: carry.clone(),
            u1: Xor::new(a.clone(), b.clone(), sum.clone()),
            u2: And::new(a.clone(), b.clone(), carry.clone()),
        }
    }

    pub fn eval(&mut self) {
        self.u1.eval();
        self.u2.eval();
    }
}

