use verilog2rust::rhdl::prelude::*;

#[derive(Debug, Clone)]
pub struct FullAdder {
    pub a: WireRef,
    pub b: WireRef,
    pub cin: WireRef,
    pub sum: WireRef,
    pub cout: WireRef,
    s: WireRef,
    c1: WireRef,
    c2: WireRef,
    u1: Xor,
    u2: Xor,
    u3: And,
    u4: And,
    u5: Or,
}

impl FullAdder {
    pub fn new(
        a: WireRef,
        b: WireRef,
        cin: WireRef,
        sum: WireRef,
        cout: WireRef,
    ) -> Self {
        let s = wire("s");
        let c1 = wire("c1");
        let c2 = wire("c2");
        FullAdder {
            a: a.clone(),
            b: b.clone(),
            cin: cin.clone(),
            sum: sum.clone(),
            cout: cout.clone(),
            s: s.clone(),
            c1: c1.clone(),
            c2: c2.clone(),
            u1: Xor::new(a.clone(), b.clone(), s.clone()),
            u2: Xor::new(s.clone(), cin.clone(), sum.clone()),
            u3: And::new(a.clone(), b.clone(), c1.clone()),
            u4: And::new(s.clone(), cin.clone(), c2.clone()),
            u5: Or::new(c1.clone(), c2.clone(), cout.clone()),
        }
    }

    pub fn eval(&mut self) {
        self.u1.eval();
        self.u2.eval();
        self.u3.eval();
        self.u4.eval();
        self.u5.eval();
    }
}

