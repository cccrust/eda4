use verilog2rust::rhdl::prelude::*;

#[derive(Debug, Clone)]
pub struct Decoder2x4 {
    pub en: WireRef,
    pub a: Vec<WireRef>,
    pub y: Vec<WireRef>,
}

impl Decoder2x4 {
    pub fn new(
        en: WireRef,
        a: Vec<WireRef>,
        y: Vec<WireRef>,
    ) -> Self {
        Decoder2x4 {
            en: en.clone(),
            a: a.clone(),
            y: y.clone(),
        }
    }

    pub fn eval(&mut self) {
    if get(&self.en).not() != Level::L {
        u16_to_bus(&self.y, (0 & 15u64) as u16);
    } else {
        let __case_val = bus_to_u16(&self.a) as u64;
        if __case_val == 0 {
            u16_to_bus(&self.y, (1 & 15u64) as u16);
        }
        if __case_val == 1 {
            u16_to_bus(&self.y, (2 & 15u64) as u16);
        }
        if __case_val == 2 {
            u16_to_bus(&self.y, (4 & 15u64) as u16);
        }
        if __case_val == 3 {
            u16_to_bus(&self.y, (8 & 15u64) as u16);
        }
    }
    }
}

