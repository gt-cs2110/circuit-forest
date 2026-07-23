mod splitter;

use crate::bitarray::BitArray;
use crate::engine::CircuitGraphMap;
use crate::engine::func::{BitSize, Component, PortProperties, PortType, PortUpdate, RunContext, port_list};
pub use splitter::*;

/// An input.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Input {
    bitsize: BitSize
}
impl Input {
    /// Creates a new instance of the tri-state buffer with specified bitsize.
    pub fn new(bitsize: u8) -> Self {
        Self {
            bitsize: BitSize::new_clamped(bitsize)
        }
    }

    /// Gets the bitsize of this component.
    pub fn get_bitsize(&self) -> u8 {
        self.bitsize.get()
    }
}
impl Component for Input {
    fn ports(&self, _: &CircuitGraphMap) -> Vec<PortProperties> {
        port_list(&[
            // output
            (PortProperties { ty: PortType::Output, bitsize: self.bitsize.get() }, 1),
        ])
    }

    fn run_inner(&self, _ctx: RunContext<'_>) -> Vec<PortUpdate> {
        vec![]
    }
}

/// An output.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Output {
    bitsize: BitSize
}
impl Output {
    /// Creates a new instance of the tri-state buffer with specified bitsize.
    pub fn new(bitsize: u8) -> Self {
        Self {
            bitsize: BitSize::new_clamped(bitsize)
        }
    }

    /// Gets the bitsize of this component.
    pub fn get_bitsize(&self) -> u8 {
        self.bitsize.get()
    }
}
impl Component for Output {
    fn ports(&self, _: &CircuitGraphMap) -> Vec<PortProperties> {
        port_list(&[
            // output
            (PortProperties { ty: PortType::Input, bitsize: self.bitsize.get() }, 1),
        ])
    }

    fn run_inner(&self, _ctx: RunContext<'_>) -> Vec<PortUpdate> {
        vec![]
    }
}

/// A constant.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Constant {
    value: BitArray
}
impl Constant {
    /// Creates a new instance of the tri-state buffer with specified bitsize.
    pub fn new(value: BitArray) -> Self {
        Self { value }
    }

    /// Gets the value which this constant holds.
    pub fn get_value(&self) -> BitArray {
        self.value
    }
}
impl Component for Constant {
    fn ports(&self, _: &CircuitGraphMap) -> Vec<PortProperties> {
        port_list(&[
            // output
            (PortProperties { ty: PortType::Output, bitsize: self.value.len() }, 1),
        ])
    }

    fn initialize_port_state(&self, state: &mut [BitArray]) {
        state[0] = self.value;
    }
    fn run_inner(&self, _ctx: RunContext<'_>) -> Vec<PortUpdate> {
        vec![]
    }
}
