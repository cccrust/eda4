use verilog2rust::rhdl::prelude::*;

fn main() {
    println!("=== Logic Gate Demo ===\n");

    let a = wire("a");
    let b = wire("b");
    let y_and = wire("y_and");
    let y_or = wire("y_or");
    let y_xor = wire("y_xor");

    let mut and = And::new(a.clone(), b.clone(), y_and.clone());
    let mut or = Or::new(a.clone(), b.clone(), y_or.clone());
    let mut xor = Xor::new(a.clone(), b.clone(), y_xor.clone());

    println!(" a b | AND OR XOR");
    println!("-----+-----------");

    for &av in &[Level::L, Level::H] {
        for &bv in &[Level::L, Level::H] {
            set(&a, av);
            set(&b, bv);
            and.eval();
            or.eval();
            xor.eval();

            println!(
                " {} {} |  {}   {}   {}",
                if av == Level::H { "1" } else { "0" },
                if bv == Level::H { "1" } else { "0" },
                if get(&y_and) == Level::H { "1" } else { "0" },
                if get(&y_or) == Level::H { "1" } else { "0" },
                if get(&y_xor) == Level::H { "1" } else { "0" },
            );
        }
    }
}
