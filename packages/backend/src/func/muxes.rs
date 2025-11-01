
use crate::func::{Component, PortProperties, PortType, PortUpdate, port_list};
use crate::{bitarr, bitarray::BitArray};

/// Minimum number of selector bits for Mux/Demux/Decoder.
pub const MIN_SELSIZE: u8 = 1;
/// Maximum number of selector bits for Mux/Demux/Decoder.
pub const MAX_SELSIZE: u8 = 6;

/// A multiplexer (mux) component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Mux {
    bitsize: u8,
    selsize: u8
}
impl Mux {
    /// Creates a new instance of the Mux with specified bitsize and selector size.
    pub fn new(bitsize: u8, selsize: u8) -> Self {
        Self {
            bitsize: bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE),
            selsize: selsize.clamp(MIN_SELSIZE, MAX_SELSIZE)
        }
    }
}
impl Component for Mux {
    fn ports(&self) -> Vec<PortProperties> {
        port_list(&[
            // selector
            (PortProperties { ty: PortType::Input, bitsize: self.selsize }, 1),
            // inputs
            (PortProperties { ty: PortType::Input, bitsize: self.bitsize }, 1 << self.selsize),
            // output
            (PortProperties { ty: PortType::Output, bitsize: self.bitsize }, 1),
        ])
    }

    fn run_inner(&self, _old_ports: &[BitArray], new_ports: &[BitArray]) -> Vec<PortUpdate> {
        let m_sel = u64::try_from(new_ports[0]);
        let result = match m_sel {
            Ok(sel) => new_ports[sel as usize + 1],
            Err(e) => BitArray::repeat(e.bit_state(), self.bitsize),
        };
        vec![PortUpdate { index: (1 << self.selsize) + 1, value: result }]
    }
}

/// A demultiplexer (demux) component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Demux {
    bitsize: u8,
    selsize: u8
}
impl Demux {
    /// Creates a new instance of the Demux with specified bitsize and selector size.
    pub fn new(bitsize: u8, selsize: u8) -> Self {
        Self {
            bitsize: bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE),
            selsize: selsize.clamp(MIN_SELSIZE, MAX_SELSIZE)
        }
    }
}
impl Component for Demux {
    fn ports(&self) -> Vec<PortProperties> {
            port_list(&[
            // selector
            (PortProperties { ty: PortType::Input, bitsize: self.selsize }, 1),
            // input
            (PortProperties { ty: PortType::Input, bitsize: self.bitsize }, 1),
            // outputs
            (PortProperties { ty: PortType::Output, bitsize: self.bitsize }, 1 << self.selsize),
        ])
    }
    fn run_inner(&self, _old_ports: &[BitArray], new_ports: &[BitArray]) -> Vec<PortUpdate> {
        let m_sel = u64::try_from(new_ports[0]);
        let result = match m_sel {
            Ok(sel) => {
                let mut result = vec![bitarr![0; self.bitsize]; 1 << self.selsize];
                result[sel as usize] = new_ports[1];
                result
            },
            Err(e) => vec![BitArray::repeat(e.bit_state(), self.bitsize); 1 << self.selsize],
        };

        result.into_iter()
            .enumerate()
            .map(|(i, value)| PortUpdate { index: 2 + i, value })
            .collect()
    }
}

/// A decoder component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Decoder {
    selsize: u8
}
impl Decoder {
    /// Creates a new instance of the Decoder with specified selector size.
    pub fn new(selsize: u8) -> Self {
        Self {
            selsize: selsize.clamp(MIN_SELSIZE, MAX_SELSIZE)
        }
    }
}
impl Component for Decoder {
    fn ports(&self) -> Vec<PortProperties> {
        port_list(&[
            // selector
            (PortProperties { ty: PortType::Input, bitsize: self.selsize }, 1),
            // outputs
            (PortProperties { ty: PortType::Output, bitsize: 1 }, 1 << self.selsize),
        ])
    }

    fn run_inner(&self, _old_ports: &[BitArray], new_ports: &[BitArray]) -> Vec<PortUpdate> {
        let m_sel = u64::try_from(new_ports[0]);
        let result = match m_sel {
            Ok(sel) => {
                let mut result = vec![bitarr![0]; 1 << self.selsize];
                result[sel as usize] = bitarr![1];
                result
            },
            Err(e) => vec![BitArray::from(e.bit_state()); 1 << self.selsize],
        };

        result.into_iter()
            .enumerate()
            .map(|(i, value)| PortUpdate { index: 1 + i, value })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bitarray::BitArray, func::floating_ports};

    #[test]
    fn test_mux() {
        const BITSIZE: u8 = 4;
        // use all possible selector sizes
        for selsize in MIN_SELSIZE..=MAX_SELSIZE {
            // 2^selsize *data* inputs
            let input_count = 1 << selsize;

            // create mux
            let mux = Mux::new(BITSIZE, selsize);
            let props = mux.ports();

            assert_eq!(props.len(), input_count + 2, "Mux with selsize {selsize} should have {} ports", input_count + 2);
            assert_eq!(props[0], PortProperties { ty: PortType::Input, bitsize: selsize }, "First Mux port should be an input selector of bitsize {selsize}");
            assert_eq!(props[input_count + 1], PortProperties { ty: PortType::Output, bitsize: BITSIZE }, "Last Mux port should be an output of bitsize {BITSIZE}");
            assert_eq!(
                props[1..=input_count],
                vec![PortProperties { ty: PortType::Input, bitsize: BITSIZE }; input_count],
                "Mux with selsize {selsize} should have {input_count} input ports"
            );

            let mut ports = floating_ports(&props);
            // Set mux inputs to random-ish values
            for (i, p) in std::iter::zip(0.., &mut ports[1..=input_count]) {
                let value = (i + 1) * 13;

                let result = p.replace(BitArray::from_bits(value, BITSIZE));
                assert!(result.is_ok());
            }

            // test all possible selector values
            for sel in 0..input_count {
                // Update selector
                let result = ports[0].replace(BitArray::from_bits(sel as u64, selsize));
                assert!(result.is_ok());

                let actual = mux.run(&ports, &ports);
                let expected = vec![PortUpdate { index: 1 + input_count, value: ports[1 + sel] }];

                assert_eq!(
                    actual,
                    expected,
                    "Mux with selsize {selsize} and selector {sel} should output correct value"
                )
            }
        }
    }

    #[test]
    fn test_demux() {
        const BITSIZE: u8 = 4;
        // use all possible selector sizes
        for selsize in MIN_SELSIZE..=MAX_SELSIZE {
            // 2^selsize *data* inputs
            let input_count = 1 << selsize;

            // create demux
            let demux = Demux::new(BITSIZE, selsize);
            let props = demux.ports();

            assert_eq!(props.len(), input_count + 2, "Demux with selsize {selsize} should have {} ports", input_count + 2);
            assert_eq!(props[0], PortProperties { ty: PortType::Input, bitsize: selsize }, "First Demux port should be an input selector of bitsize {selsize}");
            assert_eq!(props[1], PortProperties { ty: PortType::Input, bitsize: BITSIZE }, "Last Demux port should be an output of bitsize {BITSIZE}");
            assert_eq!(
                props[2..=input_count + 1],
                vec![PortProperties { ty: PortType::Output, bitsize: BITSIZE }; input_count],
                "Demux with selsize {selsize} should have {input_count} output ports"
            );

            // inputs are random-ish values
            let inputs: Vec<BitArray> = (0..input_count)
                .map(|i| (i + 1) * 13)
                .map(|val| BitArray::from_bits(val as u64, BITSIZE))
                .collect();

            let mut ports = floating_ports(&props);
            // test all possible selector values
            for (sel, &expected_out) in inputs.iter().enumerate() {
                assert!(ports[0].replace(BitArray::from_bits(sel as u64, selsize)).is_ok());
                assert!(ports[1].replace(expected_out).is_ok());

                let actual = demux.run(&ports, &ports);
                let expected: Vec<_> = (0..input_count).map(|i| PortUpdate {
                    index: 2 + i,
                    // Outputs should update so that everything is 0000,
                    // except for the selected output.
                    value: if i == sel {
                        inputs[sel]
                    } else {
                        bitarr![0; BITSIZE]
                    }
                })
                .collect();

                assert_eq!(
                    actual,
                    expected,
                    "Demux with selsize {selsize} and selector {sel} should output correct value"
                );
            }
        }
    }

    #[test]
    fn test_decoder() {
        // use all possible selector sizes
        for selsize in MIN_SELSIZE..=MAX_SELSIZE {
            // 2^selsize outputs
            let output_count = 1 << selsize;

            // create decoder
            let decoder = Decoder::new(selsize);
            let props = decoder.ports();

            assert_eq!(props.len(), output_count + 1, "Decoder with selsize {selsize} should have {} ports", output_count + 1);
            assert_eq!(props[0], PortProperties { ty: PortType::Input, bitsize: selsize }, "First Decoder port should be an input selector of bitsize {selsize}");
            assert_eq!(
                props[1..],
                vec![PortProperties { ty: PortType::Output, bitsize: 1 }; output_count],
                "Decoder with selsize {selsize} should have {output_count} output ports"
            );

            let mut ports = floating_ports(&props);
            // test all possible selector values
            for sel in 0..output_count {
                let result = ports[0].replace(BitArray::from_bits(sel as u64, selsize));
                assert!(result.is_ok());
                
                let actual = decoder.run(&ports, &ports);
                let expected = (0..output_count).map(|i| PortUpdate {
                    index: 1 + i,
                    // Only selected index should be lit
                    value: if i == sel {
                            bitarr![1]
                        } else {
                            bitarr![0]
                        }
                    })
                    .collect::<Vec<_>>();

                assert_eq!(
                    actual,
                    expected,
                    "Decoder with selsize {selsize} and selector {sel} should output correct value"
                );
            }
        }
    }
}