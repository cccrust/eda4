// MCU0m CPU — 直接使用 rhdl 模擬
// 計算 1+2+...+10 = 55

use verilog2rust::rhdl::prelude::*;

struct Cpu {
    clk: WireRef,
    a: Vec<WireRef>,   // 16-bit accumulator
    ir: Vec<WireRef>,  // 16-bit instruction register
    sw: Vec<WireRef>,  // 16-bit status word
    pc: Vec<WireRef>,  // 16-bit program counter
    pc0: Vec<WireRef>, // old PC
    m: Vec<Vec<WireRef>>, // memory array: 33 x 8-bit
    t: Vec<WireRef>,   // time counter (16-bit)
}

impl Cpu {
    pub fn new(clk: WireRef) -> Self {
        let a = bus("a", 16);
        let ir = bus("ir", 16);
        let sw = bus("sw", 16);
        let pc = bus("pc", 16);
        let pc0 = bus("pc0", 16);
        let mut m = Vec::new();
        for i in 0..33 {
            m.push(bus(&format!("m{}", i), 8));
        }
        let t = bus("t", 16);
        Self { clk, a, ir, sw, pc, pc0, m, t }
    }

    pub fn init(&mut self) {
        u16_to_bus(&mut self.pc, 0);
        u16_to_bus(&mut self.sw, 0);
        u16_to_bus(&mut self.t, 0);

        // 載入機器碼 mcu0m.hex
        let prog: [u8; 28] = [
            0x00, 0x16,  // 00  LOOP: LD   I
            0x40, 0x1A,  // 02        CMP  N
            0x50, 0x12,  // 04        JEQ  EXIT
            0x10, 0x18,  // 06        ADD  K1
            0x30, 0x16,  // 08        ST   I
            0x00, 0x14,  // 0A        LD   SUM
            0x10, 0x16,  // 0C        ADD  I
            0x30, 0x14,  // 0E        ST   SUM
            0x20, 0x00,  // 10        JMP  LOOP
            0x20, 0x12,  // 12  EXIT: JMP  EXIT
            0x00, 0x00,  // 14  SUM:  WORD 0
            0x00, 0x00,  // 16  I:    WORD 0
            0x00, 0x01,  // 18  K1:   WORD 1
            0x00, 0x0A,  // 1A  N:    WORD 10
        ];
        for i in 0..28 {
            u16_to_bus(&mut self.m[i], prog[i] as u16);
        }

        // 印出記憶體內容 ({m[i], m[i+1]} → big-endian)
        println!("Memory dump:");
        for i in (0..28).step_by(2) {
            let v = ((bus_to_u16(&self.m[i]) as u64) << 8) |
                    (bus_to_u16(&self.m[i+1]) as u64);
            println!("{:8x}: {:8x}", i, v);
        }
    }

    pub fn eval(&mut self) {
        // 指令擷取
        let pc_val = bus_to_u16(&self.pc) as usize;
        // {m[PC], m[PC+1]} — big-endian
        let ir_high = bus_to_u16(&self.m[pc_val]);
        let ir_low  = bus_to_u16(&self.m[pc_val + 1]);
        let ir_val = ((ir_high as u16) << 8) | (ir_low as u16);
        u16_to_bus(&mut self.ir, ir_val);

        let old_pc = bus_to_u16(&self.pc);
        u16_to_bus(&mut self.pc0, old_pc);
        u16_to_bus(&mut self.pc, old_pc + 2);

        let op = (ir_val >> 12) & 0xF;
        let c = (ir_val & 0xFFF) as usize;
        // {m[C], m[C+1]} — big-endian
        let m_high = bus_to_u16(&self.m[c]);
        let m_low  = bus_to_u16(&self.m[c+1]);
        let m_val  = ((m_high as u16) << 8) | (m_low as u16);
        let a_val = bus_to_u16(&self.a);

        let (new_a, new_sw, jump) = if op == 0 {
            (m_val, bus_to_u16(&self.sw), false)
        } else if op == 3 {
            // {m[C], m[C+1]} = A — big-endian
            u16_to_bus(&mut self.m[c], (a_val >> 8) & 0xFF);
            u16_to_bus(&mut self.m[c+1], a_val & 0xFF);
            (a_val, bus_to_u16(&self.sw), false)
        } else if op == 4 {
            let n = if (a_val as i16) < (m_val as i16) { 1 } else { 0 };
            let z = if a_val == m_val { 1 } else { 0 };
            (a_val, (n << 15) | (z << 14), false)
        } else if op == 1 {
            let sum = (a_val as u16).wrapping_add(m_val);
            (sum, bus_to_u16(&self.sw), false)
        } else if op == 2 {
            (a_val, bus_to_u16(&self.sw), true)
        } else if op == 5 {
            let z = (bus_to_u16(&self.sw) >> 14) & 1;
            (a_val, bus_to_u16(&self.sw), z == 1)
        } else {
            (a_val, bus_to_u16(&self.sw), false)
        };

        u16_to_bus(&mut self.a, new_a);
        u16_to_bus(&mut self.sw, new_sw);
        if jump {
            u16_to_bus(&mut self.pc, c as u16);
        }

        let time_ns = bus_to_u16(&self.t);
        let t = time_ns as u64;
        println!("{:4}ns PC={:04x} IR={:04x}, SW={:04x}, A={}", t, old_pc, ir_val, bus_to_u16(&self.sw), bus_to_u16(&self.a) as i16);
        u16_to_bus(&mut self.t, time_ns + 10);
    }
}

fn main() {
    let clk = wire("clk");
    let mut cpu = Cpu::new(clk.clone());
    cpu.init();

    // 模擬 100 個 clock 週期
    for _cycle in 0..100 {
        set(&clk, Level::H);
        cpu.eval();
        set(&clk, Level::L);
    }
}
