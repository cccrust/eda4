use verilog2rust::rhdl::prelude::*;

#[derive(Debug, Clone)]
pub struct Mux2 {
    pub a: WireRef,
    pub b: WireRef,
    pub sel: WireRef,
    pub y: WireRef,
    s: WireRef,
    not_sel: WireRef,
    t1: WireRef,
    t2: WireRef,
    u1: Not,
    u2: And,
    u3: And,
    u4: Or,
}

impl Mux2 {
    pub fn new(
        a: WireRef,
        b: WireRef,
        sel: WireRef,
        y: WireRef,
    ) -> Self {
        let s = wire("s");
        let not_sel = wire("not_sel");
        let t1 = wire("t1");
        let t2 = wire("t2");
        Mux2 {
            a: a.clone(),
            b: b.clone(),
            sel: sel.clone(),
            y: y.clone(),
            s: s.clone(),
            not_sel: not_sel.clone(),
            t1: t1.clone(),
            t2: t2.clone(),
            u1: Not::new(sel.clone(), not_sel.clone()),
            u2: And::new(a.clone(), not_sel.clone(), t1.clone()),
            u3: And::new(b.clone(), sel.clone(), t2.clone()),
            u4: Or::new(t1.clone(), t2.clone(), y.clone()),
        }
    }

    pub fn eval(&mut self) {
        self.u1.eval();
        self.u2.eval();
        self.u3.eval();
        self.u4.eval();
    }
}

