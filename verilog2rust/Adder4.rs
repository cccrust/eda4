use verilog2rust::rhdl::prelude::*;

#[derive(Debug, Clone)]
pub struct Adder4 {
    pub a: Vec<WireRef>,
    pub b: Vec<WireRef>,
    pub cin: WireRef,
    pub sum: Vec<WireRef>,
    pub cout: WireRef,
    c: Vec<WireRef>,
    fa0: FullAdder,
    fa1: FullAdder,
    fa2: FullAdder,
    fa3: FullAdder,
}

impl Adder4 {
    pub fn new(
        a: Vec<WireRef>,
        b: Vec<WireRef>,
        cin: WireRef,
        sum: Vec<WireRef>,
        cout: WireRef,
    ) -> Self {
        let c = bus("c", 4);
        Adder4 {
            a: a.clone(),
            b: b.clone(),
            cin: cin.clone(),
            sum: sum.clone(),
            cout: cout.clone(),
            c: c.clone(),
            fa0: FullAdder::new(a[0].clone(), b[0].clone(), cin.clone(), sum[0].clone(), c[0].clone()),
            fa1: FullAdder::new(a[1].clone(), b[1].clone(), c[0].clone(), sum[1].clone(), c[1].clone()),
            fa2: FullAdder::new(a[2].clone(), b[2].clone(), c[1].clone(), sum[2].clone(), c[2].clone()),
            fa3: FullAdder::new(a[3].clone(), b[3].clone(), c[2].clone(), sum[3].clone(), cout.clone()),
        }
    }

    pub fn eval(&mut self) {
        self.fa0.eval();
        self.fa1.eval();
        self.fa2.eval();
        self.fa3.eval();
    }
}

