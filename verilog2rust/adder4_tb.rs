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

#[derive(Debug, Clone)]
pub struct Adder4Tb {
    sum: Vec<WireRef>,
    cout: WireRef,
    a: Vec<WireRef>,
    b: Vec<WireRef>,
    cin: WireRef,
    dut: Adder4,
}

impl Adder4Tb {
    pub fn new(
    ) -> Self {
        let sum = bus("sum", 4);
        let cout = wire("cout");
        let a = bus("a", 4);
        let b = bus("b", 4);
        let cin = wire("cin");
        Adder4Tb {
            sum: sum.clone(),
            cout: cout.clone(),
            a: a.clone(),
            b: b.clone(),
            cin: cin.clone(),
            dut: Adder4::new(a.clone(), b.clone(), cin.clone(), sum.clone(), cout.clone()),
        }
    }

    pub fn eval(&mut self) {
        self.dut.eval();
    }
    pub fn run(&mut self) {
        println!("=== Adder4 Testbench ===");
        u16_to_bus(&self.a, (3 & 15u64) as u16);
        u16_to_bus(&self.b, (5 & 15u64) as u16);
        if get(&self.cin) != Level::L { set(&self.cin, Level::L); }
        self.eval();
        println!("3 + 5 + 0 = {} (carry={})", bus_to_u16(&self.sum) as u64, get(&self.cout) as u64);
        u16_to_bus(&self.a, (9 & 15u64) as u16);
        u16_to_bus(&self.b, (7 & 15u64) as u16);
        if get(&self.cin) != Level::H { set(&self.cin, Level::H); }
        self.eval();
        println!("9 + 7 + 1 = {} (carry={})", bus_to_u16(&self.sum) as u64, get(&self.cout) as u64);
        u16_to_bus(&self.a, (0 & 15u64) as u16);
        u16_to_bus(&self.b, (15 & 15u64) as u16);
        if get(&self.cin) != Level::L { set(&self.cin, Level::L); }
        self.eval();
        println!("0 + 15 + 0 = {} (carry={})", bus_to_u16(&self.sum) as u64, get(&self.cout) as u64);
        u16_to_bus(&self.a, (15 & 15u64) as u16);
        u16_to_bus(&self.b, (15 & 15u64) as u16);
        if get(&self.cin) != Level::H { set(&self.cin, Level::H); }
        self.eval();
        println!("15 + 15 + 1 = {} (carry={})", bus_to_u16(&self.sum) as u64, get(&self.cout) as u64);
        return;
    }
}

fn main() {
    let mut tb = Adder4Tb::new();
    tb.run();
}
