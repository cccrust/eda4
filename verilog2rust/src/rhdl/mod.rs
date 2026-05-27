pub mod signal;
pub mod gate;
pub mod sim;

pub mod prelude {
    pub use crate::rhdl::gate::{And, Nand, Nor, Not, Or, Xor};
    pub use crate::rhdl::signal::{
        bits_to_u64, bus, bus_to_u16, get, get_bus, set, set_bus, u16_to_bus, val_to_bits, wire,
        Level, WireRef,
    };
    pub use crate::rhdl::sim::Sim;
}
