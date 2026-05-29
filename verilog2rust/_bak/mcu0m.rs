use verilog2rust::rhdl::prelude::*;

#[derive(Debug, Clone)]
pub struct Cpu {
    pub clock: WireRef,
    a: Vec<WireRef>,
    i_r: Vec<WireRef>,
    s_w: Vec<WireRef>,
    p_c: Vec<WireRef>,
    pc0: Vec<WireRef>,
    m: Vec<Vec<WireRef>>,
    i: i64,
    sim_time: u64,
    running: bool,
    _prev_clock: Level,
}

impl Cpu {
    pub fn new(
        clock: WireRef,
    ) -> Self {
        let a = bus("a", 16);
        let i_r = bus("i_r", 16);
        let s_w = bus("s_w", 16);
        let p_c = bus("p_c", 16);
        let pc0 = bus("pc0", 16);
        let m = (0..33).map(|_| bus("m", 8)).collect::<Vec<Vec<WireRef>>>();
        Cpu {
            clock: clock.clone(),
            a: a.clone(),
            i_r: i_r.clone(),
            s_w: s_w.clone(),
            p_c: p_c.clone(),
            pc0: pc0.clone(),
            m: m.clone(),
            i: 0,
            sim_time: 0u64,
            running: true,
            _prev_clock: Level::L,
        }
    }

    pub fn eval(&mut self) {
    self.sim_time += 1;
    let _posedge_clock = self._prev_clock == Level::L && get(&self.clock) == Level::H;
    let _negedge_clock = self._prev_clock == Level::H && get(&self.clock) == Level::L;
    self._prev_clock = get(&self.clock);
    if _posedge_clock {
        u16_to_bus(&self.i_r, ((((bus_to_u16(&self.m[(((bus_to_u16(&self.p_c) as u64) + (1)) & 65535u64) as usize]) as u64) & 255u64) << 0) | (((bus_to_u16(&self.m[(bus_to_u16(&self.p_c) as u64) as usize]) as u64) & 255u64) << 8) & 65535u64) as u16);
        u16_to_bus(&self.pc0, (bus_to_u16(&self.p_c) as u64 & 65535u64) as u16);
        u16_to_bus(&self.p_c, (((bus_to_u16(&self.p_c) as u64) + (2)) & 65535u64 & 65535u64) as u16);
        let __case_val = (bus_to_u16(&self.i_r) as u64 >> 12) & 15u64;
        if __case_val == 0u64 {
            u16_to_bus(&self.a, ((((bus_to_u16(&self.m[((((bus_to_u16(&self.i_r) as u64 >> 0) & 4095u64) + (1)) & 4095u64) as usize]) as u64) & 255u64) << 0) | (((bus_to_u16(&self.m[((bus_to_u16(&self.i_r) as u64 >> 0) & 4095u64) as usize]) as u64) & 255u64) << 8) & 65535u64) as u16);
        }
        if __case_val == 3u64 {
            let __concat_val = bus_to_u16(&self.a) as u64;
            for __j in 0..8 {
                let __b = (__concat_val >> (8 + __j)) & 1;
                set(&self.m[((bus_to_u16(&self.i_r) as u64 >> 0) & 4095u64) as usize][__j], if __b == 1 { Level::H } else { Level::L });
            }
            for __j in 0..8 {
                let __b = (__concat_val >> (0 + __j)) & 1;
                set(&self.m[((((bus_to_u16(&self.i_r) as u64 >> 0) & 4095u64) + (1)) & 4095u64) as usize][__j], if __b == 1 { Level::H } else { Level::L });
            }
        }
        if __case_val == 4u64 {
            if get(&self.s_w[15]) != if (bus_to_u16(&self.a) as u64) < ((((bus_to_u16(&self.m[((((bus_to_u16(&self.i_r) as u64 >> 0) & 4095u64) + (1)) & 4095u64) as usize]) as u64) & 255u64) << 0) | (((bus_to_u16(&self.m[((bus_to_u16(&self.i_r) as u64 >> 0) & 4095u64) as usize]) as u64) & 255u64) << 8)) { Level::H } else { Level::L } { set(&self.s_w[15], if (bus_to_u16(&self.a) as u64) < ((((bus_to_u16(&self.m[((((bus_to_u16(&self.i_r) as u64 >> 0) & 4095u64) + (1)) & 4095u64) as usize]) as u64) & 255u64) << 0) | (((bus_to_u16(&self.m[((bus_to_u16(&self.i_r) as u64 >> 0) & 4095u64) as usize]) as u64) & 255u64) << 8)) { Level::H } else { Level::L }); }
            if get(&self.s_w[14]) != if (bus_to_u16(&self.a) as u64) == ((((bus_to_u16(&self.m[((((bus_to_u16(&self.i_r) as u64 >> 0) & 4095u64) + (1)) & 4095u64) as usize]) as u64) & 255u64) << 0) | (((bus_to_u16(&self.m[((bus_to_u16(&self.i_r) as u64 >> 0) & 4095u64) as usize]) as u64) & 255u64) << 8)) { Level::H } else { Level::L } { set(&self.s_w[14], if (bus_to_u16(&self.a) as u64) == ((((bus_to_u16(&self.m[((((bus_to_u16(&self.i_r) as u64 >> 0) & 4095u64) + (1)) & 4095u64) as usize]) as u64) & 255u64) << 0) | (((bus_to_u16(&self.m[((bus_to_u16(&self.i_r) as u64 >> 0) & 4095u64) as usize]) as u64) & 255u64) << 8)) { Level::H } else { Level::L }); }
        }
        if __case_val == 1u64 {
            u16_to_bus(&self.a, (((bus_to_u16(&self.a) as u64) + ((((bus_to_u16(&self.m[((((bus_to_u16(&self.i_r) as u64 >> 0) & 4095u64) + (1)) & 4095u64) as usize]) as u64) & 255u64) << 0) | (((bus_to_u16(&self.m[((bus_to_u16(&self.i_r) as u64 >> 0) & 4095u64) as usize]) as u64) & 255u64) << 8))) & 65535u64 & 65535u64) as u16);
        }
        if __case_val == 2u64 {
            u16_to_bus(&self.p_c, ((bus_to_u16(&self.i_r) as u64 >> 0) & 4095u64 & 65535u64) as u16);
        }
        if __case_val == 5u64 {
            if (bus_to_u16(&self.s_w) as u64 >> 14) & 1 != 0 {
                u16_to_bus(&self.p_c, ((bus_to_u16(&self.i_r) as u64 >> 0) & 4095u64 & 65535u64) as u16);
            }
        }
        println!("{:>4}ns PC={:x} IR={:x}, SW={:x}, A={}", self.sim_time as u64, bus_to_u16(&self.pc0) as u64, bus_to_u16(&self.i_r) as u64, bus_to_u16(&self.s_w) as u64, bus_to_u16(&self.a) as u64);
    }
    }
    pub fn run(&mut self) {
        u16_to_bus(&self.p_c, (0 & 65535u64) as u16);
        u16_to_bus(&self.s_w, (0 & 65535u64) as u16);
        let __data = std::fs::read_to_string("mcu0m.hex").unwrap_or_else(|_| "".to_string());
        let __lines: Vec<&str> = __data.lines().collect();
        let mut __addr = 0usize;
        let mut __in_block = false;
        for __line in &__lines {
            let __line = __line.trim();
            if __line.is_empty() || __line.starts_with("//") || __line.starts_with('#') { continue; }
            let __comment_pos = __line.find("//").unwrap_or(__line.len());
            let __line = &__line[..__comment_pos].trim();
            if __line.is_empty() { continue; }
            if __line.starts_with('@') {
                __addr = usize::from_str_radix(&__line[1..], 16).unwrap_or(0);
                __in_block = true;
                continue;
            }
            if !__in_block { __in_block = true; }
            for __token in __line.split_whitespace() {
                if let Ok(__val) = u64::from_str_radix(__token.trim_start_matches("0x").trim_start_matches("0X"), 16) {
                    let __elem_mask = (1u64 << 8) - 1;
                    u16_to_bus(&mut self.m[__addr], (__val & __elem_mask) as u16);
                    __addr += 1;
                }
            }
        }
        self.i = (0) as i64;
        while if (self.i as u64) < (32) { 1 } else { 0 } != 0 {
            println!("{:>8x}: {:>8x}", self.i as u64, (((bus_to_u16(&self.m[(((self.i as u64) + (1)) & 65535u64) as usize]) as u64) & 255u64) << 0) | (((bus_to_u16(&self.m[(self.i as u64) as usize]) as u64) & 255u64) << 8));
            self.i = (((self.i as u64) + (2)) & 65535u64) as i64;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Main {
    clock: WireRef,
    cpux: Cpu,
    sim_time: u64,
    running: bool,
}

impl Main {
    pub fn new(
    ) -> Self {
        let clock = wire("clock");
        Main {
            clock: clock.clone(),
            cpux: Cpu::new(clock.clone()),
            sim_time: 0u64,
            running: true,
        }
    }

    pub fn eval(&mut self) {
    self.sim_time += 1;
        self.cpux.eval();
    }
    pub fn run(&mut self) {
        self.cpux.run();
        if get(&self.clock) != Level::L { set(&self.clock, Level::L); }
        for _ in 0..200 {
            if !self.running { break; }
            for _ in 0..10 { self.eval(); }
            if get(&self.clock) != get(&self.clock).not() { set(&self.clock, get(&self.clock).not()); }
            self.eval();
        }
        self.running = false;
        return;
    }
}

fn main() {
    let mut tb = Main::new();
    tb.run();
}
