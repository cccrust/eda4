use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

static WIRE_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    L,
    H,
    X,
    Z,
}

impl Level {
    pub fn from_bool(b: bool) -> Self {
        if b { Level::H } else { Level::L }
    }

    pub fn to_bool(self) -> Option<bool> {
        match self {
            Level::H => Some(true),
            Level::L => Some(false),
            _ => None,
        }
    }

    pub fn not(self) -> Self {
        match self {
            Level::L => Level::H,
            Level::H => Level::L,
            Level::X => Level::X,
            Level::Z => Level::X,
        }
    }

    pub fn and(self, other: Self) -> Self {
        if self == Level::L || other == Level::L {
            Level::L
        } else if self == Level::H && other == Level::H {
            Level::H
        } else {
            Level::X
        }
    }

    pub fn or(self, other: Self) -> Self {
        if self == Level::H || other == Level::H {
            Level::H
        } else if self == Level::L && other == Level::L {
            Level::L
        } else {
            Level::X
        }
    }

    pub fn xor(self, other: Self) -> Self {
        if self == Level::X || other == Level::X || self == Level::Z || other == Level::Z {
            Level::X
        } else if self != other {
            Level::H
        } else {
            Level::L
        }
    }

    pub fn nand(self, other: Self) -> Self {
        self.and(other).not()
    }

    pub fn nor(self, other: Self) -> Self {
        self.or(other).not()
    }
}

impl From<bool> for Level {
    fn from(b: bool) -> Self {
        Level::from_bool(b)
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Level::L => write!(f, "0"),
            Level::H => write!(f, "1"),
            Level::X => write!(f, "X"),
            Level::Z => write!(f, "Z"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Wire {
    pub value: Level,
    pub name: String,
}

impl Wire {
    pub fn new(name: &str) -> Self {
        Wire { value: Level::X, name: name.to_string() }
    }
}

pub type WireRef = Rc<RefCell<Wire>>;

pub fn wire(name: &str) -> WireRef {
    let n = WIRE_COUNTER.fetch_add(1, Ordering::Relaxed);
    Rc::new(RefCell::new(Wire::new(&format!("{}_{}", name, n))))
}

pub fn bus(name: &str, width: usize) -> Vec<WireRef> {
    (0..width).map(|i| wire(&format!("{}{}", name, i))).collect()
}

pub fn get(w: &WireRef) -> Level {
    w.borrow().value
}

pub fn set(w: &WireRef, val: Level) {
    w.borrow_mut().value = val;
}

pub fn set_bus(b: &[WireRef], vals: &[Level]) {
    for (i, w) in b.iter().enumerate() {
        if i < vals.len() {
            set(w, vals[i]);
        }
    }
}

pub fn get_bus(b: &[WireRef]) -> Vec<Level> {
    b.iter().map(|w| get(w)).collect()
}

pub fn bus_to_u16(b: &[WireRef]) -> u16 {
    let mut val = 0u16;
    for (i, w) in b.iter().enumerate() {
        if get(w) == Level::H {
            val |= 1 << i;
        }
    }
    val
}

pub fn u16_to_bus(b: &[WireRef], val: u16) {
    for (i, w) in b.iter().enumerate() {
        let bit = (val >> i) & 1;
        set(w, if bit == 1 { Level::H } else { Level::L });
    }
}

pub fn val_to_bits(val: u64, width: usize) -> Vec<Level> {
    (0..width).map(|i| {
        if (val >> i) & 1 == 1 { Level::H } else { Level::L }
    }).collect()
}

pub fn bits_to_u64(bits: &[Level]) -> u64 {
    let mut val = 0u64;
    for (i, &bit) in bits.iter().enumerate() {
        if bit == Level::H {
            val |= 1 << i;
        }
    }
    val
}
