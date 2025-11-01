use crate::bitarray::BitArray;
use crate::circuit::CircuitGraphMap;
use crate::func::{Component, PortProperties, PortType, PortUpdate, RunContext, Sensitivity, port_list};

/// An input.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Input {
    bitsize: u8
}
impl Input {
    /// Creates a new instance of the tri-state buffer with specified bitsize.
    pub fn new(bitsize: u8) -> Self {
        Self {
            bitsize: bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE)
        }
    }
}
impl Component for Input {
    fn ports(&self, _: &CircuitGraphMap) -> Vec<PortProperties> {
        port_list(&[
            // output
            (PortProperties { ty: PortType::Output, bitsize: self.bitsize }, 1),
        ])
    }

    fn run_inner(&self, _ctx: RunContext<'_>) -> Vec<PortUpdate> {
        vec![]
    }
}

/// An output.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Output {
    bitsize: u8
}
impl Output {
    /// Creates a new instance of the tri-state buffer with specified bitsize.
    pub fn new(bitsize: u8) -> Self {
        Self {
            bitsize: bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE)
        }
    }
}
impl Component for Output {
    fn ports(&self, _: &CircuitGraphMap) -> Vec<PortProperties> {
        port_list(&[
            // output
            (PortProperties { ty: PortType::Input, bitsize: self.bitsize }, 1),
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
    bitsize: u8
}
impl Splitter {
    /// Creates a new instance of the Splitter with specified bitsize.
    pub fn new(bitsize: u8) -> Self {
        Self {
            bitsize: bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE)
        }
    }
}
impl Component for Splitter {
    fn ports(&self, _: &CircuitGraphMap) -> Vec<PortProperties> {
        port_list(&[
            // joined
            (PortProperties { ty: PortType::Inout, bitsize: self.bitsize }, 1),
            // split
            (PortProperties { ty: PortType::Inout, bitsize: 1 }, self.bitsize),
        ])
    }

    fn run_inner(&self, ctx: RunContext<'_>) -> Vec<PortUpdate> {
        if Sensitivity::Anyedge.activated(ctx.old_ports[0], ctx.new_ports[0]) {
            std::iter::zip(1.., ctx.new_ports[0])
                .map(|(index, bit)| PortUpdate { index, value: BitArray::from(bit) })
                .collect()
        } else if Sensitivity::Anyedge.any_activated(&ctx.old_ports[1..], &ctx.new_ports[1..]) {
            let value = ctx.new_ports[1..].iter()
                .map(|b| b.index(0))
                .collect();
            vec![PortUpdate { index: 0, value }]
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bitarray::{BitArray, BitState}, func::floating_ports};

    #[test]
    fn test_splitter_split() {
        for bitsize in BitArray::MIN_BITSIZE..=BitArray::MAX_BITSIZE {
            let splitter = Splitter::new(bitsize);
            let props = splitter.ports(&Default::default());

            assert_eq!(props.len(), 1 + bitsize as usize, "Splitter with bitsize {bitsize} should have {} ports", 1 + bitsize as usize);
            assert_eq!(props[0], PortProperties { ty: PortType::Inout, bitsize }, "First Splitter port should be an inout of bitsize {bitsize}");
            assert_eq!(
                props[1..],
                vec![PortProperties { ty: PortType::Inout, bitsize: 1 }; bitsize as usize],
                "Splitter with bitsize {bitsize} should have {bitsize} split ports"
            );

            let old_ports = floating_ports(&props);
            let mut new_ports = floating_ports(&props);
            
            let data: Vec<_> = (0..bitsize)
                .map(|i| match i % 2 == 0 {
                    true => BitState::High,
                    false => BitState::Low
                }).collect();
            
            let joined = BitArray::from_iter(data.iter().copied());
            let result = new_ports[0].replace(joined);
            assert!(result.is_ok());
            
            let actual = splitter.run(RunContext {
                graphs: &Default::default(),
                old_ports: &old_ports, 
                new_ports: &new_ports,
                inner_state: None
            });
            let expected: Vec<_> = data.iter().enumerate()
                .map(|(i, &st)| PortUpdate {
                    index: 1 + i,
                    value: BitArray::from(st)
                })
                .collect();

            assert_eq!(
                actual,
                expected,
                "Splitter should correctly split the input bits"
            );
        }
    }

    #[test]
    fn test_splitter_join() {
        for bitsize in BitArray::MIN_BITSIZE..=BitArray::MAX_BITSIZE {
            let splitter = Splitter::new(bitsize);
            let props = splitter.ports(&Default::default());

            assert_eq!(props.len(), 1 + bitsize as usize, "Splitter with bitsize {bitsize} should have {} ports", 1 + bitsize as usize);
            assert_eq!(props[0], PortProperties { ty: PortType::Inout, bitsize }, "First Splitter port should be an inout of bitsize {bitsize}");
            assert_eq!(
                props[1..],
                vec![PortProperties { ty: PortType::Inout, bitsize: 1 }; bitsize as usize],
                "Splitter with bitsize {bitsize} should have {bitsize} split ports"
            );

            let old_ports = floating_ports(&props);
            let mut new_ports = floating_ports(&props);
            
            let data: Vec<_> = (0..bitsize)
                .map(|i| match i % 2 == 0 {
                    true => BitState::High,
                    false => BitState::Low
                }).collect();
            
            let joined = BitArray::from_iter(data.iter().copied());
            let split: Vec<_> = data.into_iter()
                .map(BitArray::from)
                .collect();
            for (p, arr) in std::iter::zip(new_ports[1..].iter_mut(), split) {
                let result = p.replace(arr);
                assert!(result.is_ok());
            }
            
            let actual = splitter.run(RunContext {
                graphs: &Default::default(),
                old_ports: &old_ports, 
                new_ports: &new_ports,
                inner_state: None
            });
            let expected = vec![PortUpdate { index: 0, value: joined }];

            assert_eq!(
                actual,
                expected,
                "Splitter should correctly split the input bits"
            );
        }
    }
}
