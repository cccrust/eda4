use crate::rhdl::signal::{get, set, wire, Level, WireRef};
use std::cell::RefCell;
use std::rc::Rc;

type EvalFn = Rc<RefCell<dyn FnMut()>>;

pub struct Sim {
    comb: Vec<EvalFn>,
    seq: Vec<EvalFn>,
    pub clk: WireRef,
    time: u64,
}

impl Sim {
    pub fn new() -> Self {
        Sim {
            comb: Vec::new(),
            seq: Vec::new(),
            clk: wire("clk"),
            time: 0,
        }
    }

    pub fn add_comb<F: FnMut() + 'static>(&mut self, mut f: F) {
        self.comb.push(Rc::new(RefCell::new(move || f())));
    }

    pub fn add_seq<F: FnMut() + 'static>(&mut self, mut f: F) {
        self.seq.push(Rc::new(RefCell::new(move || f())));
    }

    pub fn eval(&mut self) {
        for _ in 0..10 {
            let mut changed = false;
            for f in &self.comb {
                let orig = get(&self.clk);
                f.borrow_mut()();
                if get(&self.clk) != orig {
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
    }

    pub fn posedge(&mut self) {
        set(&self.clk, Level::H);
        for f in &self.seq {
            f.borrow_mut()();
        }
        self.eval();
    }

    pub fn negedge(&mut self) {
        set(&self.clk, Level::L);
        for f in &self.seq {
            f.borrow_mut()();
        }
    }

    pub fn tick(&mut self) {
        self.posedge();
        self.negedge();
        self.time += 1;
    }

    pub fn run(&mut self, cycles: u64) {
        for _ in 0..cycles {
            self.tick();
        }
    }

    pub fn reset(&mut self) {
        set(&self.clk, Level::L);
        self.time = 0;
    }

    pub fn time(&self) -> u64 {
        self.time
    }

    pub fn get_clock(&self) -> WireRef {
        self.clk.clone()
    }
}

impl Default for Sim {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rhdl::gate::{And, Not, Or, Xor};

    #[test]
    fn test_sim_comb() {
        let mut sim = Sim::new();
        let a = wire("a"); let b = wire("b"); let y = wire("y");
        let mut gate = And::new(a.clone(), b.clone(), y.clone());
        sim.add_comb(move || gate.eval());

        set(&a, Level::H); set(&b, Level::H);
        sim.eval();
        assert_eq!(get(&y), Level::H);
    }

    #[test]
    fn test_sim_multiple_gates() {
        let mut sim = Sim::new();
        let a = wire("a"); let b = wire("b"); let cin = wire("cin");
        let xor_ab = wire("xor_ab");
        let s = wire("s");
        let c1 = wire("c1"); let c2 = wire("c2"); let cout = wire("cout");

        let mut x1 = Xor::new(a.clone(), b.clone(), xor_ab.clone());
        let mut x2 = Xor::new(xor_ab.clone(), cin.clone(), s.clone());
        let mut a1 = And::new(a.clone(), b.clone(), c1.clone());
        let mut a2 = And::new(xor_ab.clone(), cin.clone(), c2.clone());
        let mut o1 = Or::new(c1.clone(), c2.clone(), cout.clone());

        sim.add_comb(move || { x1.eval(); x2.eval(); a1.eval(); a2.eval(); o1.eval(); });

        set(&a, Level::H); set(&b, Level::H); set(&cin, Level::H);
        sim.eval();
        assert_eq!(get(&s), Level::H);
        assert_eq!(get(&cout), Level::H);

        set(&a, Level::H); set(&b, Level::L); set(&cin, Level::L);
        sim.eval();
        assert_eq!(get(&s), Level::H);
        assert_eq!(get(&cout), Level::L);
    }

    #[test]
    fn test_sim_convergence() {
        let mut sim = Sim::new();
        let a = wire("a"); let b = wire("b");
        let not_a = wire("not_a");
        let mut n1 = Not::new(a.clone(), not_a.clone());
        let mut n2 = Not::new(not_a.clone(), b.clone());
        sim.add_comb(move || { n1.eval(); n2.eval(); });

        set(&a, Level::H);
        sim.eval();
        assert_eq!(get(&b), Level::H);
    }
}
