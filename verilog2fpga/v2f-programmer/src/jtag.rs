/// JTAG TAP (Test Access Port) 狀態機
///
/// IEEE 1149.1 JTAG 標準的 16 狀態 Finite State Machine。
/// 狀態轉換由 TMS (Test Mode Select) 訊號控制。

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TapState {
    TestLogicReset,
    RunTestIdle,
    SelectDRScan,
    CaptureDR,
    ShiftDR,
    Exit1DR,
    PauseDR,
    Exit2DR,
    UpdateDR,
    SelectIRScan,
    CaptureIR,
    ShiftIR,
    Exit1IR,
    PauseIR,
    Exit2IR,
    UpdateIR,
}

impl TapState {
    pub fn transition(self, tms: bool) -> Self {
        match self {
            TapState::TestLogicReset => if tms { TapState::TestLogicReset } else { TapState::RunTestIdle },
            TapState::RunTestIdle => if tms { TapState::SelectDRScan } else { TapState::RunTestIdle },
            TapState::SelectDRScan => if tms { TapState::SelectIRScan } else { TapState::CaptureDR },
            TapState::CaptureDR => if tms { TapState::Exit1DR } else { TapState::ShiftDR },
            TapState::ShiftDR => if tms { TapState::Exit1DR } else { TapState::ShiftDR },
            TapState::Exit1DR => if tms { TapState::UpdateDR } else { TapState::PauseDR },
            TapState::PauseDR => if tms { TapState::Exit2DR } else { TapState::PauseDR },
            TapState::Exit2DR => if tms { TapState::UpdateDR } else { TapState::ShiftDR },
            TapState::UpdateDR => if tms { TapState::SelectDRScan } else { TapState::RunTestIdle },
            TapState::SelectIRScan => if tms { TapState::TestLogicReset } else { TapState::CaptureIR },
            TapState::CaptureIR => if tms { TapState::Exit1IR } else { TapState::ShiftIR },
            TapState::ShiftIR => if tms { TapState::Exit1IR } else { TapState::ShiftIR },
            TapState::Exit1IR => if tms { TapState::UpdateIR } else { TapState::PauseIR },
            TapState::PauseIR => if tms { TapState::Exit2IR } else { TapState::PauseIR },
            TapState::Exit2IR => if tms { TapState::UpdateIR } else { TapState::ShiftIR },
            TapState::UpdateIR => if tms { TapState::SelectDRScan } else { TapState::RunTestIdle },
        }
    }

    pub fn is_shift(&self) -> bool {
        matches!(self, TapState::ShiftDR | TapState::ShiftIR)
    }
}

#[derive(Debug, Clone)]
pub struct JtagStateMachine {
    pub state: TapState,
    pub tms_buffer: Vec<bool>,
    pub tdi_buffer: Vec<u8>,
    pub tdo_buffer: Vec<u8>,
}

impl Default for JtagStateMachine {
    fn default() -> Self {
        JtagStateMachine {
            state: TapState::TestLogicReset,
            tms_buffer: Vec::new(),
            tdi_buffer: Vec::new(),
            tdo_buffer: Vec::new(),
        }
    }
}

impl JtagStateMachine {
    pub fn new() -> Self { Self::default() }

    pub fn reset(&mut self) { self.state = TapState::TestLogicReset; }

    pub fn advance(&mut self, tms: bool, tdi: bool) -> bool {
        self.state = self.state.transition(tms);
        self.tms_buffer.push(tms);
        self.tdi_buffer.push(if tdi { 1 } else { 0 });
        self.tdo_buffer.push(0);
        false
    }

    pub fn go_to(&mut self, target: TapState) {
        let path = shortest_path(self.state, target);
        for tms in path {
            self.advance(tms, false);
        }
    }

    pub fn shift_dr(&mut self, data: &[u8], _tdi_last: bool) -> Vec<u8> {
        self.go_to(TapState::ShiftDR);
        let tdo = Vec::new();
        let last_bit = data.len().saturating_sub(1);
        for (i, &byte) in data.iter().enumerate() {
            for bit in 0..8 {
                let is_last = i == last_bit && bit == 7;
                let _tms = is_last;
                let _tdi_val = ((byte >> bit) & 1) != 0;
                self.state = if is_last { self.state.transition(true) } else { self.state.transition(false) };
            }
        }
        self.go_to(TapState::RunTestIdle);
        tdo
    }

    pub fn shift_ir(&mut self, instruction: u8, len: u8) {
        self.go_to(TapState::ShiftIR);
        for bit in 0..len {
            let is_last = bit == len - 1;
            let _tdi_val = ((instruction >> bit) & 1) != 0;
            self.state = self.state.transition(is_last);
        }
        self.go_to(TapState::RunTestIdle);
    }
}

fn shortest_path(from: TapState, to: TapState) -> Vec<bool> {
    use TapState::*;
    let all_states = [
        TestLogicReset, RunTestIdle, SelectDRScan, CaptureDR, ShiftDR,
        Exit1DR, PauseDR, Exit2DR, UpdateDR, SelectIRScan, CaptureIR,
        ShiftIR, Exit1IR, PauseIR, Exit2IR, UpdateIR,
    ];
    let mut best: Option<Vec<bool>> = None;
    let mut stack = vec![(from, Vec::new())];
    let mut visited = std::collections::HashSet::new();
    while let Some((s, path)) = stack.pop() {
        if s == to { best = Some(path); break; }
        if !visited.insert(s) { continue; }
        if path.len() > 10 { continue; }
        stack.push((s.transition(true), { let mut p = path.clone(); p.push(true); p }));
        stack.push((s.transition(false), { let mut p = path.clone(); p.push(false); p }));
    }
    best.unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let jtag = JtagStateMachine::new();
        assert_eq!(jtag.state, TapState::TestLogicReset);
    }

    #[test]
    fn test_reset_stays_in_reset() {
        let mut jtag = JtagStateMachine::new();
        jtag.advance(true, false);
        assert_eq!(jtag.state, TapState::TestLogicReset);
    }

    #[test]
    fn test_reset_to_idle() {
        let mut jtag = JtagStateMachine::new();
        jtag.advance(false, false);
        assert_eq!(jtag.state, TapState::RunTestIdle);
    }

    #[test]
    fn test_shift_dr_path() {
        let mut jtag = JtagStateMachine::new();
        jtag.go_to(TapState::ShiftDR);
        assert_eq!(jtag.state, TapState::ShiftDR);
    }

    #[test]
    fn test_shift_ir_path() {
        let mut jtag = JtagStateMachine::new();
        jtag.go_to(TapState::ShiftIR);
        assert_eq!(jtag.state, TapState::ShiftIR);
    }

    #[test]
    fn test_all_state_transitions() {
        let states = [
            TapState::TestLogicReset, TapState::RunTestIdle,
            TapState::SelectDRScan, TapState::CaptureDR, TapState::ShiftDR,
            TapState::Exit1DR, TapState::PauseDR, TapState::Exit2DR, TapState::UpdateDR,
            TapState::SelectIRScan, TapState::CaptureIR, TapState::ShiftIR,
            TapState::Exit1IR, TapState::PauseIR, TapState::Exit2IR, TapState::UpdateIR,
        ];
        for &tms in &[true, false] {
            for &from in &states {
                let to = from.transition(tms);
                assert!(states.contains(&to), "無效轉換 {from:?} --({tms})--> {to:?}");
            }
        }
    }

    #[test]
    fn test_shift_dr_instruction() {
        let mut jtag = JtagStateMachine::new();
        jtag.shift_ir(0x02, 5);
        assert_eq!(jtag.state, TapState::RunTestIdle);
    }
}
