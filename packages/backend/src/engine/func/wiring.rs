use crate::bitarray::BitArray;
use crate::engine::CircuitGraphMap;
use crate::engine::func::{BitSize, Component, PortBit, PortProperties, PortType, PortUpdate, RunContext, port_list};

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

/// A splitter component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Splitter {
    bitsize: BitSize
}
impl Splitter {
    /// Creates a new instance of the Splitter with specified bitsize.
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
impl Component for Splitter {
    fn ports(&self, _: &CircuitGraphMap) -> Vec<PortProperties> {
        port_list(&[
            // joined
            (PortProperties { ty: PortType::Passive, bitsize: self.bitsize.get() }, 1),
            // split
            (PortProperties { ty: PortType::Passive, bitsize: 1 }, usize::from(self.bitsize)),
        ])
    }

    fn passive_links(&self) -> Vec<(PortBit, PortBit)> {
        (0..self.bitsize.get())
            .map(|bit| (PortBit { index: 0, bit }, PortBit { index: 1 + usize::from(bit), bit: 0 }))
            .collect()
    }

    fn run_inner(&self, _ctx: RunContext<'_>) -> Vec<PortUpdate> {
        // values are directly updated in value node propagations w/ the passive ports
        // so there's no updates here
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_splitter_split() {
        for bitsize in BitArray::MIN_BITSIZE..=BitArray::MAX_BITSIZE {
            let splitter = Splitter::new(bitsize);
            let props = splitter.ports(&Default::default());

            assert_eq!(props.len(), 1 + bitsize as usize, "Splitter with bitsize {bitsize} should have {} ports", 1 + bitsize as usize);
            assert_eq!(props[0], PortProperties { ty: PortType::Passive, bitsize }, "First Splitter port should be a passive port of bitsize {bitsize}");
            assert_eq!(
                props[1..],
                vec![PortProperties { ty: PortType::Passive, bitsize: 1 }; bitsize as usize],
                "Splitter with bitsize {bitsize} should have {bitsize} split ports"
            );
        }
    }

    #[test]
    fn test_splitter_links() {
        for bitsize in BitArray::MIN_BITSIZE..=BitArray::MAX_BITSIZE {
            let splitter = Splitter::new(bitsize);
            let expected: Vec<_> = (0..bitsize)
                .map(|bit| (PortBit { index: 0, bit }, PortBit { index: 1 + usize::from(bit), bit: 0 }))
                .collect();

            assert_eq!(splitter.passive_links(), expected, "Splitter should link joined bit i to split port i");
        }
    }
}
