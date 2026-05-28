use crate::rhdl::signal::{get, set, WireRef};

macro_rules! binary_gate {
    ($name:ident, $op:ident) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            pub a: Vec<WireRef>,
            pub y: WireRef,
        }

        impl $name {
            pub fn new(a: Vec<WireRef>, y: WireRef) -> Self {
                $name { a, y }
            }

            pub fn eval(&mut self) {
                let mut v = get(&self.a[0]);
                for i in 1..self.a.len() {
                    v = v.$op(get(&self.a[i]));
                }
                if get(&self.y) != v {
                    set(&self.y, v);
                }
            }
        }
    };
}

binary_gate!(And, and);
binary_gate!(Or, or);
binary_gate!(Xor, xor);
binary_gate!(Nand, nand);
binary_gate!(Nor, nor);

#[derive(Debug, Clone)]
pub struct Not {
    pub a: Vec<WireRef>,
    pub y: WireRef,
}

impl Not {
    pub fn new(a: Vec<WireRef>, y: WireRef) -> Self {
        Not { a, y }
    }

    pub fn eval(&mut self) {
        let v = get(&self.a[0]).not();
        if get(&self.y) != v {
            set(&self.y, v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rhdl::signal::{wire, Level};

    #[test]
    fn test_not() {
        let a = wire("a");
        let y = wire("y");
        let mut g = Not::new(vec![a.clone()], y.clone());
        set(&a, Level::L); g.eval(); assert_eq!(get(&y), Level::H);
        set(&a, Level::H); g.eval(); assert_eq!(get(&y), Level::L);
        set(&a, Level::X); g.eval(); assert_eq!(get(&y), Level::X);
    }

    #[test]
    fn test_and() {
        let a = wire("a"); let b = wire("b"); let y = wire("y");
        let mut g = And::new(vec![a.clone(), b.clone()], y.clone());
        for &av in &[Level::L, Level::H] {
            for &bv in &[Level::L, Level::H] {
                set(&a, av); set(&b, bv); g.eval();
                let expected = av.and(bv);
                assert_eq!(get(&y), expected, "AND {:?} {:?}", av, bv);
            }
        }
    }

    #[test]
    fn test_or() {
        let a = wire("a"); let b = wire("b"); let y = wire("y");
        let mut g = Or::new(vec![a.clone(), b.clone()], y.clone());
        for &av in &[Level::L, Level::H] {
            for &bv in &[Level::L, Level::H] {
                set(&a, av); set(&b, bv); g.eval();
                assert_eq!(get(&y), av.or(bv));
            }
        }
    }

    #[test]
    fn test_xor() {
        let a = wire("a"); let b = wire("b"); let y = wire("y");
        let mut g = Xor::new(vec![a.clone(), b.clone()], y.clone());
        for &av in &[Level::L, Level::H] {
            for &bv in &[Level::L, Level::H] {
                set(&a, av); set(&b, bv); g.eval();
                assert_eq!(get(&y), av.xor(bv));
            }
        }
    }

    #[test]
    fn test_nand() {
        let a = wire("a"); let b = wire("b"); let y = wire("y");
        let mut g = Nand::new(vec![a.clone(), b.clone()], y.clone());
        for &av in &[Level::L, Level::H] {
            for &bv in &[Level::L, Level::H] {
                set(&a, av); set(&b, bv); g.eval();
                assert_eq!(get(&y), av.nand(bv));
            }
        }
    }

    #[test]
    fn test_nor() {
        let a = wire("a"); let b = wire("b"); let y = wire("y");
        let mut g = Nor::new(vec![a.clone(), b.clone()], y.clone());
        for &av in &[Level::L, Level::H] {
            for &bv in &[Level::L, Level::H] {
                set(&a, av); set(&b, bv); g.eval();
                assert_eq!(get(&y), av.nor(bv));
            }
        }
    }
}
