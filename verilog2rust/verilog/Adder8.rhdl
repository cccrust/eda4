use verilog2rust::rhdl::prelude::*;

#[derive(Debug, Clone)]
pub struct Adder8 {
    pub a: Vec<WireRef>,
    pub b: Vec<WireRef>,
    pub cin: WireRef,
    pub sum: Vec<WireRef>,
    pub cout: WireRef,
    c4: WireRef,
    low: Adder4,
    high: Adder4,
}

impl Adder8 {
    pub fn new(
        a: Vec<WireRef>,
        b: Vec<WireRef>,
        cin: WireRef,
        sum: Vec<WireRef>,
        cout: WireRef,
    ) -> Self {
        let c4 = wire("c4");
        Adder8 {
            a: a.clone(),
            b: b.clone(),
            cin: cin.clone(),
            sum: sum.clone(),
            cout: cout.clone(),
            c4: c4.clone(),
            low: Adder4::new(vec![a[0], a[1], a[2], a[3]].clone(), vec![b[0], b[1], b[2], b[3]].clone(), cin.clone(), vec![sum[0], sum[1], sum[2], sum[3]].clone(), c4.clone()),
            high: Adder4::new(vec![a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7]].clone(), vec![b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]].clone(), c4.clone(), vec![sum[0], sum[1], sum[2], sum[3], sum[4], sum[5], sum[6], sum[7]].clone(), cout.clone()),
        }
    }

    pub fn eval(&mut self) {
        self.low.eval();
        self.high.eval();
    }
}

