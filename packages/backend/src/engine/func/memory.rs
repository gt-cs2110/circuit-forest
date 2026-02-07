use crate::bitarr;
use crate::bitarray::{BitArray, BitState};
use crate::engine::CircuitGraphMap;
use crate::engine::func::{Component, PortProperties, PortType, PortUpdate, RunContext, Sensitivity, port_list};

/// A register component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Register {
    bitsize: u8
}
impl Register {
    /// Creates a new instance of the Register with specified bitsize.
    pub fn new(bitsize: u8) -> Self {
        Self {
            bitsize: bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE)
        }
    }
}
impl Component for Register {
    fn ports(&self, _: &CircuitGraphMap) -> Vec<PortProperties> {
        port_list(&[
            // din
            (PortProperties { ty: PortType::Input, bitsize: self.bitsize }, 1),
            // enable, clock, clear
            (PortProperties { ty: PortType::Input, bitsize: 1 }, 3),
            // dout
            (PortProperties { ty: PortType::Output, bitsize: self.bitsize }, 1),
        ])
    }
    fn initialize_port_state(&self, state: &mut [BitArray]) {
        state[4] = bitarr![0; self.bitsize];
    }
    fn run_inner(&self, ctx: RunContext<'_>) -> Vec<PortUpdate> {
        if ctx.new_ports[3].all(BitState::High) {
            vec![PortUpdate { index: 4, value: bitarr![0; self.bitsize] }]
        } else if Sensitivity::Posedge.activated(ctx.old_ports[2], ctx.new_ports[2]) && ctx.new_ports[1].all(BitState::High) {
            vec![PortUpdate { index: 4, value: ctx.new_ports[0] }]
        } else {
            vec![]
        }
    }
}
