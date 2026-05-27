pub type BitId = u64;

#[derive(Debug, Clone)]
pub struct Netlist {
    pub top: String,
    pub ports: Vec<NetPort>,
    pub cells: Vec<Cell>,
    pub bits: Vec<BitInfo>,
    pub next_bit: BitId,
}

#[derive(Debug, Clone)]
pub struct NetPort {
    pub name: String,
    pub direction: PortDir,
    pub bits: Vec<BitId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortDir {
    Input, Output, Inout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CellKind {
    Input, Output,
    And, Or, Xor, Not,
    Dff,
    Lut { init: u16 },
    Carry,
    Mux2,
    Const0, Const1,
    Add, Sub,
}

#[derive(Debug, Clone)]
pub struct Cell {
    pub name: String,
    pub kind: CellKind,
    pub inputs: Vec<(String, Vec<BitId>)>,
    pub outputs: Vec<(String, Vec<BitId>)>,
}

#[derive(Debug, Clone)]
pub struct BitInfo {
    pub id: BitId,
    pub name: Option<String>,
    #[allow(dead_code)]
    pub is_port: bool,
}

impl Netlist {
    pub fn new(top: &str) -> Self {
        Netlist {
            top: top.to_string(),
            ports: Vec::new(),
            cells: Vec::new(),
            bits: Vec::new(),
            next_bit: 1,
        }
    }

    pub fn alloc_bit(&mut self) -> BitId {
        let id = self.next_bit;
        self.next_bit += 1;
        self.bits.push(BitInfo { id, name: None, is_port: false });
        id
    }

    pub fn alloc_bits(&mut self, n: u32) -> Vec<BitId> {
        (0..n).map(|_| self.alloc_bit()).collect()
    }

    pub fn name_bit(&mut self, id: BitId, name: &str) {
        if let Some(b) = self.bits.iter_mut().find(|b| b.id == id) {
            b.name = Some(name.to_string());
        }
    }

    pub fn add_cell(&mut self, kind: CellKind, inputs: Vec<(&str, Vec<BitId>)>, outputs: Vec<(&str, Vec<BitId>)>) {
        let name = format!("${}", self.cells.len());
        self.cells.push(Cell {
            name,
            kind,
            inputs: inputs.into_iter().map(|(n, b)| (n.to_string(), b)).collect(),
            outputs: outputs.into_iter().map(|(n, b)| (n.to_string(), b)).collect(),
        });
    }
}
