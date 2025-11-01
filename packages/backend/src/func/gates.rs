use crate::bitarray::{BitArray, BitState, bitarr};
use crate::func::{Component, PortProperties, PortType, PortUpdate, port_list};

/// Minimum number of inputs for multi-input logic gates.
pub const MIN_GATE_INPUTS: u8 = 2;
/// Maximum number of inputs for multi-input logic gates.
pub const MAX_GATE_INPUTS: u8 = 64;

macro_rules! gates {
    ($($(#[$m:meta])? $Id:ident: $f:expr, $invert:literal),*$(,)?) => {
        $(
            $(#[$m])?
            #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
            pub struct $Id {
                bitsize: u8,
                n_inputs: u8
            }
            impl $Id {
                /// Creates a new instance of the gate with specified bitsize and number of inputs.
                pub fn new(bitsize: u8, n_inputs: u8) -> Self {
                    Self {
                        bitsize: bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE),
                        n_inputs: n_inputs.clamp(MIN_GATE_INPUTS, MAX_GATE_INPUTS)
                    }
                }
            }
            impl Component for $Id {
                fn ports(&self) -> Vec<PortProperties> {
                    port_list(&[
                        // inputs
                        (PortProperties { ty: PortType::Input, bitsize: self.bitsize }, self.n_inputs),
                        // outputs
                        (PortProperties { ty: PortType::Output, bitsize: self.bitsize }, 1),
                    ])
                }
                fn run_inner(&self, _old_ports: &[BitArray], new_ports: &[BitArray]) -> Vec<PortUpdate> {
                    let value = new_ports[..usize::from(self.n_inputs)].iter()
                        .cloned()
                        .reduce($f)
                        .unwrap_or_else(|| bitarr![X; self.bitsize]);
    
                    vec![PortUpdate {
                        index: usize::from(self.n_inputs),
                        value: if $invert { !value } else { value }
                    }]
                }
            }
        )*
    }
}

gates! {
    /// An AND gate component.
    And:  |a, b| a & b, false,
    /// An OR gate component.
    Or:   |a, b| a | b, false,
    /// An XOR gate component.
    Xor:  |a, b| a ^ b, false,
    /// A NAND gate component.
    Nand: |a, b| a & b, true,
    /// A NOR gate component.
    Nor:  |a, b| a | b, true,
    /// A XNOR gate component.
    Xnor: |a, b| a ^ b, true,
}

/// A NOT gate component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Not {
    bitsize: u8
}
impl Not {
    /// Creates a new instance of the NOT gate with specified bitsize.
    pub fn new(bitsize: u8) -> Self {
        Self {
            bitsize: bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE)
        }
    }
}
impl Component for Not {
    fn ports(&self) -> Vec<PortProperties> {
        port_list(&[
            // input
            (PortProperties { ty: PortType::Input, bitsize: self.bitsize }, 1),
            // output
            (PortProperties { ty: PortType::Output, bitsize: self.bitsize }, 1),
        ])
    }

    fn run_inner(&self, _old_ports: &[BitArray], new_ports: &[BitArray]) -> Vec<PortUpdate> {
        vec![PortUpdate { index: 1, value: !new_ports[0] }]
    }
}


/// A tri-state buffer component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct TriState {
    bitsize: u8
}
impl TriState {
    /// Creates a new instance of the tri-state buffer with specified bitsize.
    pub fn new(bitsize: u8) -> Self {
        Self {
            bitsize: bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE)
        }
    }
}
impl Component for TriState {
    fn ports(&self) -> Vec<PortProperties> {
        port_list(&[
            // selector
            (PortProperties { ty: PortType::Input, bitsize: 1 }, 1),
            // input
            (PortProperties { ty: PortType::Input, bitsize: self.bitsize }, 1),
            // output
            (PortProperties { ty: PortType::Output, bitsize: self.bitsize }, 1),
        ])
    }

    fn run_inner(&self, _old_ports: &[BitArray], new_ports: &[BitArray]) -> Vec<PortUpdate> {
        let gate = new_ports[0].index(0);
        let result = match gate {
            BitState::High => new_ports[1],
            BitState::Low | BitState::Imped => bitarr![Z; self.bitsize],
            BitState::Unk => bitarr![X; self.bitsize],
        };
        vec![PortUpdate { index: 2, value: result }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_and_gate() {
        let gate = And::new(1, 2);
        let in_a = bitarr![0];
        let in_b = bitarr![1];

        let updates = gate.run(&[], &[in_a, in_b]);

        // Checks if we have only one update. Should be 1 for logic gates
        // Checks if port 2, the output port for two input gates was updated
        // 1 & 0 = 0;
        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: bitarr![0] }],
            "Expected a single update with index=2 and value=0 (1 & 0 = 0)"
        );
    }

    #[test]
    fn test_and_gate_multi_bit() {
        let gate = And::new(4, 2);
        let in_a = bitarr![1, 0, 1, 1];
        let in_b = bitarr![1, 1, 0, 0];

        let updates = gate.run(&[], &[in_a, in_b]);

        // 1011 & 1100 = 1000;
        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: bitarr![1, 0, 0, 0] }],
            "Expected a single update with index=2 and value=1000 (1011 & 1100 = 1000)"
        );
    }

    #[test]
    fn test_and_gate_3input_4bit() {
        let gate = And::new(4, 3); // 3 inputs, 4-bit each
        let in_a = bitarr![1, 0, 1, 1];
        let in_b = bitarr![1, 1, 0, 0];
        let in_c = bitarr![1, 1, 1, 0];

        let updates = gate.run(&[], &[in_a, in_b, in_c]);

        // 1011 & 1100 & 1110 = 1000;
        assert_eq!(
            updates,
            vec![PortUpdate { index: 3, value: bitarr![1, 0, 0, 0] }],
            "Expected a single update with index=3 and value=1000 (1011 & 1100 & 1110 = 1000)"
        );
    }

    #[test]
    fn test_or_gate() {
        let gate = Or::new(1, 2);
        let in_a = bitarr![0];
        let in_b = bitarr![1];

        let updates = gate.run(&[], &[in_a, in_b]);

        // 1 | 0 = 1;
        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: bitarr![1] }],
            "Expected a single update with index=2 and value=1 (1 | 0 = 1)"
        );
    }

    #[test]
    fn test_or_gate_multi_bit() {
        let gate = Or::new(4, 2);
        let in_a = bitarr![1, 0, 1, 1];
        let in_b = bitarr![1, 1, 0, 0];

        let updates = gate.run(&[], &[in_a, in_b]);

        // 1011 | 1100 = 1111;
        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: bitarr![1, 1, 1, 1] }],
            "Expected a single update with index=2 and value=1111 (1011 | 1100 = 1111)"
        );
    }

    #[test]
    fn test_or_gate_3input_4bit() {
        let gate = Or::new(4, 3);
        let in_a = bitarr![1, 0, 1, 1];
        let in_b = bitarr![1, 1, 0, 0];
        let in_c = bitarr![0, 1, 1, 0];

        let updates = gate.run(&[], &[in_a, in_b, in_c]);

        // 1011 | 1100 | 0110 = 1111;
        assert_eq!(
            updates,
            vec![PortUpdate { index: 3, value: bitarr![1, 1, 1, 1] }],
            "Expected a single update with index=3 and value=1111 (1011 | 1100 | 0110 = 1111)"
        );
    }

    #[test]
    fn test_xor_gate() {
        let gate = Xor::new(1, 2);
        let in_a = bitarr![0];
        let in_b = bitarr![1];

        let updates = gate.run(&[], &[in_a, in_b]);

        // 1 ^ 0 = 1;
        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: bitarr![1] }],
            "Expected a single update with index=2 and value=1 (1 ^ 0 = 1)"
        );
    }

    #[test]
    fn test_xor_gate_multi_bit() {
        let gate = Xor::new(4, 2);
        let in_a = bitarr![1, 0, 1, 1];
        let in_b = bitarr![1, 1, 0, 1];

        let updates = gate.run(&[], &[in_a, in_b]);

        // 1011 ^ 1101 = 0110;
        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: bitarr![0, 1, 1, 0] }],
            "Expected a single update with index=2 and value=0110 (1011 ^ 1101 = 0110)"
        );
    }

    #[test]
    fn test_xor_gate_3input_4bit() {
        let gate = Xor::new(4, 3);
        let in_a = bitarr![1, 0, 1, 1];
        let in_b = bitarr![1, 1, 0, 1];
        let in_c = bitarr![0, 1, 1, 0];

        let updates = gate.run(&[], &[in_a, in_b, in_c]);

        // 1011 ^ 1101 ^ 0110 = 0000;
        assert_eq!(
            updates,
            vec![PortUpdate { index: 3, value: bitarr![0, 0, 0, 0] }],
            "Expected a single update with index=3 and value=0000 (1011 ^ 1101 ^ 0110 = 0000)"
        );
    }

    #[test]
    fn test_nand_gate() {
        let gate = Nand::new(1, 2);
        let in_a = bitarr![0];
        let in_b = bitarr![1];

        let updates = gate.run(&[], &[in_a, in_b]);

        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: bitarr![1] }],
            "Expected a single update with index=2 and value=1 (!(1 & 0) = 1)"
        );
    }

    #[test]
    fn test_nand_gate_multi_bit() {
        let gate = Nand::new(4, 2);
        let in_a = bitarr![1, 0, 1, 1];
        let in_b = bitarr![1, 1, 0, 1];

        let updates = gate.run(&[], &[in_a, in_b]);

        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: bitarr![0, 1, 1, 0] }],
            "Expected a single update with index=2 and value=0110 (!(1011 & 1101) = 0110)"
        );
    }

    #[test]
    fn test_nand_gate_3input_4bit() {
        let gate = Nand::new(4, 3);
        let in_a = bitarr![1, 0, 1, 1];
        let in_b = bitarr![1, 1, 0, 1];
        let in_c = bitarr![1, 1, 1, 0];

        let updates = gate.run(&[], &[in_a, in_b, in_c]);

        // !(1011 & 1101 & 1110) = 0111;
        assert_eq!(
            updates,
            vec![PortUpdate { index: 3, value: bitarr![0, 1, 1, 1] }],
            "Expected a single update with index=3 and value=0111 (!(1011 & 1101 & 1110) = 0111)"
        );
    }

    #[test]
    fn test_nor_gate() {
        let gate = Nor::new(1, 2);
        let in_a = bitarr![0];
        let in_b = bitarr![1];

        let updates = gate.run(&[], &[in_a, in_b]);

        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: bitarr![0] }],
            "Expected a single update with index=2 and value=0 (!(1 | 0) = 0)"
        );
    }

    #[test]
    fn test_nor_gate_multi_bit() {
        let gate = Nor::new(4, 2);
        let in_a = bitarr![1, 0, 1, 1];
        let in_b = bitarr![1, 1, 0, 1];

        let updates = gate.run(&[], &[in_a, in_b]);

        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: bitarr![0, 0, 0, 0] }],
            "Expected a single update with index=2 and value=0000 (!(1011 | 1101) = 0000)"
        );
    }

    #[test]
    fn test_nor_gate_3input_4bit() {
        let gate = Nor::new(4, 3);
        let in_a = bitarr![1, 0, 1, 1];
        let in_b = bitarr![1, 1, 0, 1];
        let in_c = bitarr![0, 1, 1, 0];

        let updates = gate.run(&[], &[in_a, in_b, in_c]);

        // !(1011 | 1101 | 0110) = 0000;
        assert_eq!(
            updates,
            vec![PortUpdate { index: 3, value: bitarr![0, 0, 0, 0] }],
            "Expected a single update with index=3 and value=0000 (!(1011 | 1101 | 0110) = 0000)"
        );
    }

    #[test]
    fn test_xnor_gate() {
        let gate = Xnor::new(1, 2);
        let in_a = bitarr![0];
        let in_b = bitarr![1];

        let updates = gate.run(&[], &[in_a, in_b]);

        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: bitarr![0] }],
            "Expected a single update with index=2 and value=0 (!(1 ^ 0) = 0)"
        );
    }

    #[test]
    fn test_xnor_gate_multi_bit() {
        let gate = Xnor::new(4, 2);
        let in_a = bitarr![1, 0, 1, 1];
        let in_b = bitarr![1, 1, 0, 1];

        let updates = gate.run(&[], &[in_a, in_b]);

        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: bitarr![1, 0, 0, 1] }],
            "Expected a single update with index=2 and value=1001 (!(1011 ^ 1101) = 1001)"
        );
    }

    #[test]
    fn test_xnor_gate_3input_4bit() {
        let gate = Xnor::new(4, 3);
        let in_a = bitarr![1, 0, 1, 1];
        let in_b = bitarr![1, 1, 0, 1];
        let in_c = bitarr![0, 1, 1, 0];

        let updates = gate.run(&[], &[in_a, in_b, in_c]);

        // !(1011 ^ 1101 ^ 0110) = 1111;
        assert_eq!(
            updates,
            vec![PortUpdate { index: 3, value: bitarr![1, 1, 1, 1] }],
            "Expected a single update with index=3 and value=1111 (!(1011 ^ 1101 ^ 0110) = 1111)"
        );
    }

    #[test]
    fn test_not_gate() {
        let gate = Not::new(1);
        let in_a = bitarr![0];

        let updates = gate.run(&[], &[in_a]);

        assert_eq!(
            updates,
            vec![PortUpdate { index: 1, value: bitarr![1] }],
            "Expected a single update with index=1 and value=1 (!0 = 1)"
        );
    }

    #[test]
    fn test_not_gate_multi_bit() {
        let gate = Not::new(4);
        let in_a = bitarr![1, 0, 1, 1];

        let updates = gate.run(&[], &[in_a]);

        assert_eq!(
            updates,
            vec![PortUpdate { index: 1, value: bitarr![0, 1, 0, 0] }],
            "Expected a single update with index=1 and value=0100 (!1011 = 0100)"
        );
    }

    mod input_validation {
        use super::*;

        #[test]
        #[should_panic]
        fn input_validate_and() {
            let gate = And::new(4, 2);
            // Should fail input validation
            let bad_in = bitarr![1, 1, 1];
            let good_in = bitarr![1, 0, 1, 0];
            let _ = gate.run(&[], &[bad_in, good_in]);
        } 

        #[test]
        #[should_panic]
        fn input_validate_or() {
            let gate = Or::new(4, 2);
            // Should fail input validation
            let bad_in = bitarr![1, 1, 1];
            let good_in = bitarr![1, 0, 1, 0];
            let _ = gate.run(&[], &[bad_in, good_in]);
        } 

        #[test]
        #[should_panic]
        fn input_validate_xor() {
            let gate = Xor::new(4, 2);
            // Should fail input validation
            let bad_in = bitarr![1, 1, 1];
            let good_in = bitarr![1, 0, 1, 0];
            let _ = gate.run(&[], &[bad_in, good_in]);
        } 

        #[test]
        #[should_panic]
        fn input_validate_nand() {
            let gate = Nand::new(4, 2);
            // Should fail input validation
            let bad_in = bitarr![1, 1, 1];
            let good_in = bitarr![1, 0, 1, 0];
            let _ = gate.run(&[], &[bad_in, good_in]);
        } 

        #[test]
        #[should_panic]
        fn input_validate_nor() {
            let gate = Nor::new(4, 2);
            // Should fail input validation
            let bad_in = bitarr![1, 1, 1];
            let good_in = bitarr![1, 0, 1, 0];
            let _ = gate.run(&[], &[bad_in, good_in]);
        } 

        #[test]
        #[should_panic]
        fn input_validate_xnor() {
            let gate = Xnor::new(4, 2);
            // Should fail input validation
            let bad_in = bitarr![1, 1, 1];
            let good_in = bitarr![1, 0, 1, 0];
            let _ = gate.run(&[], &[bad_in, good_in]);
        } 

        #[test]
        #[should_panic]
        fn input_validate_not() {
            let gate = Not::new(4);
            // Should fail input validation
            let bad_in = bitarr![1, 1, 1];
            let _ = gate.run(&[], &[bad_in]);
        } 

        #[test]
        #[should_panic]
        fn input_validate_tristate() {
            let gate = TriState::new(4);
            // Should fail input validation
            let bad_in = bitarr![1, 1, 1];
            let _ = gate.run(&[], &[bad_in]);
        } 
    }
}
