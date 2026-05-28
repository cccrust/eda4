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
            low: Adder4::new(vec![a[0].clone(), a[1].clone(), a[2].clone(), a[3].clone()], vec![b[0].clone(), b[1].clone(), b[2].clone(), b[3].clone()], cin.clone(), vec![sum[0].clone(), sum[1].clone(), sum[2].clone(), sum[3].clone()], c4.clone()),
            high: Adder4::new(vec![a[0].clone(), a[1].clone(), a[2].clone(), a[3].clone(), a[4].clone(), a[5].clone(), a[6].clone(), a[7].clone()], vec![b[0].clone(), b[1].clone(), b[2].clone(), b[3].clone(), b[4].clone(), b[5].clone(), b[6].clone(), b[7].clone()], c4.clone(), vec![sum[0].clone(), sum[1].clone(), sum[2].clone(), sum[3].clone(), sum[4].clone(), sum[5].clone(), sum[6].clone(), sum[7].clone()], cout.clone()),
        }
    }

    pub fn eval(&mut self) {
        self.low.eval();
        self.high.eval();
    }
    pub fn run(&mut self) {
        self.low.run();
        self.high.run();
    }
}

