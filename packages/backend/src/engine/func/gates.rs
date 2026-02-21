use crate::bitarray::{BitArray, BitState, bitarr};
use crate::engine::CircuitGraphMap;
use crate::engine::func::{Component, PortProperties, PortType, PortUpdate, RunContext, port_list};

/// Minimum number of inputs for multi-input logic gates.
pub const MIN_GATE_INPUTS: u8 = 2;
/// Maximum number of inputs for multi-input logic gates.
pub const MAX_GATE_INPUTS: u8 = 64;

/// The gate type for [`Gate`].
/// 
/// This defines the logic and appearance of the gate.
#[expect(missing_docs)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum GateKind {
    And, Or, Xor, Nand, Nor, Xnor
}
impl GateKind {
    fn reduce(self, it: impl IntoIterator<Item=BitArray>) -> Option<BitArray> {
        let it = it.into_iter();
        match self {
            GateKind::And  => it.reduce(|a, b| a & b),
            GateKind::Or   => it.reduce(|a, b| a | b),
            GateKind::Xor  => it.reduce(|a, b| a ^ b),
            GateKind::Nand => it.reduce(|a, b| a & b).map(|r| !r),
            GateKind::Nor  => it.reduce(|a, b| a | b).map(|r| !r),
            GateKind::Xnor => it.reduce(|a, b| a ^ b).map(|r| !r),
        }
    }

    /// The name of the gate
    pub fn name(self) -> &'static str {
        match self {
            GateKind::And  => "And",
            GateKind::Or   => "Or",
            GateKind::Xor  => "Xor",
            GateKind::Nand => "Nand",
            GateKind::Nor  => "Nor",
            GateKind::Xnor => "Xnor",
        }
    }
}
/// A multi-input logic gate.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Gate {
    kind: GateKind,
    bitsize: u8,
    n_inputs: u8
}
impl Gate {
    /// Creates a new instance of the gate with specified bitsize and number of inputs.
    pub fn new(kind: GateKind, bitsize: u8, n_inputs: u8) -> Self {
        Self {
            kind,
            bitsize: bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE),
            n_inputs: n_inputs.clamp(MIN_GATE_INPUTS, MAX_GATE_INPUTS)
        }
    }

    /// The gate type.
    pub fn kind(&self) -> GateKind {
        self.kind
    }
    /// Returns the number of inputs for this gate.
    pub fn n_inputs(&self) -> u8 {
        self.n_inputs
    }
    /// Returns the bitsize for this gate.
    pub fn bitsize(&self) -> u8 {
        self.bitsize
    }
}
impl Component for Gate {
    fn ports(&self, _: &CircuitGraphMap) -> Vec<PortProperties> {
        port_list(&[
            // inputs
            (PortProperties { ty: PortType::Input, bitsize: self.bitsize }, usize::from(self.n_inputs)),
            // outputs
            (PortProperties { ty: PortType::Output, bitsize: self.bitsize }, 1),
        ])
    }

    fn run_inner(&self, ctx: RunContext<'_>) -> Vec<PortUpdate> {
        let inputs = ctx.new_ports[..usize::from(self.n_inputs)].iter().cloned();
        let output = self.kind.reduce(inputs)
            .unwrap_or_else(|| bitarr![X; self.bitsize]);
        
        vec![PortUpdate {
            index: usize::from(self.n_inputs),
            value: output
        }]
    }
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
    fn ports(&self, _: &CircuitGraphMap) -> Vec<PortProperties> {
        port_list(&[
            // input
            (PortProperties { ty: PortType::Input, bitsize: self.bitsize }, 1),
            // output
            (PortProperties { ty: PortType::Output, bitsize: self.bitsize }, 1),
        ])
    }

    fn run_inner(&self, ctx: RunContext<'_>) -> Vec<PortUpdate> {
        vec![PortUpdate { index: 1, value: !ctx.new_ports[0] }]
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
    fn ports(&self, _: &CircuitGraphMap) -> Vec<PortProperties> {
        port_list(&[
            // selector
            (PortProperties { ty: PortType::Input, bitsize: 1 }, 1),
            // input
            (PortProperties { ty: PortType::Input, bitsize: self.bitsize }, 1),
            // output
            (PortProperties { ty: PortType::Output, bitsize: self.bitsize }, 1),
        ])
    }

    fn run_inner(&self, ctx: RunContext<'_>) -> Vec<PortUpdate> {
        let gate = ctx.new_ports[0].index(0);
        let result = match gate {
            BitState::High => ctx.new_ports[1],
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
        let gate = Gate::new(GateKind::And, 1, 2);
        let in_a = bitarr![0];
        let in_b = bitarr![1];

        let updates = gate.run(RunContext {
            graphs: &Default::default(),
            old_ports: &[bitarr![Z]; 3],
            new_ports: &[in_a, in_b, bitarr![Z]],
            inner_state: None
        });

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
        let gate = Gate::new(GateKind::And, 4, 2);
        let in_a = bitarr![1, 0, 1, 1];
        let in_b = bitarr![1, 1, 0, 0];

        let updates = gate.run(RunContext {
            graphs: &Default::default(),
            old_ports: &[bitarr![Z; 4]; 3],
            new_ports: &[in_a, in_b, bitarr![Z; 4]],
            inner_state: None
        });

        // 1011 & 1100 = 1000;
        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: bitarr![1, 0, 0, 0] }],
            "Expected a single update with index=2 and value=1000 (1011 & 1100 = 1000)"
        );
    }

    #[test]
    fn test_and_gate_3input_4bit() {
        let gate = Gate::new(GateKind::And, 4, 3); // 3 inputs, 4-bit each
        let in_a = bitarr![1, 0, 1, 1];
        let in_b = bitarr![1, 1, 0, 0];
        let in_c = bitarr![1, 1, 1, 0];

        let updates = gate.run(RunContext {
            graphs: &Default::default(),
            old_ports: &[bitarr![Z; 4]; 4],
            new_ports: &[in_a, in_b, in_c, bitarr![Z; 4]],
            inner_state: None
        });

        // 1011 & 1100 & 1110 = 1000;
        assert_eq!(
            updates,
            vec![PortUpdate { index: 3, value: bitarr![1, 0, 0, 0] }],
            "Expected a single update with index=3 and value=1000 (1011 & 1100 & 1110 = 1000)"
        );
    }

    #[test]
    fn test_or_gate() {
        let gate = Gate::new(GateKind::Or, 1, 2);
        let in_a = bitarr![0];
        let in_b = bitarr![1];

        let updates = gate.run(RunContext {
            graphs: &Default::default(),
            old_ports: &[bitarr![Z]; 3],
            new_ports: &[in_a, in_b, bitarr![Z]],
            inner_state: None
        });

        // 1 | 0 = 1;
        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: bitarr![1] }],
            "Expected a single update with index=2 and value=1 (1 | 0 = 1)"
        );
    }

    #[test]
    fn test_or_gate_multi_bit() {
        let gate = Gate::new(GateKind::Or, 4, 2);
        let in_a = bitarr![1, 0, 1, 1];
        let in_b = bitarr![1, 1, 0, 0];

        let updates = gate.run(RunContext {
            graphs: &Default::default(),
            old_ports: &[bitarr![Z; 4]; 3],
            new_ports: &[in_a, in_b, bitarr![Z; 4]],
            inner_state: None
        });

        // 1011 | 1100 = 1111;
        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: bitarr![1, 1, 1, 1] }],
            "Expected a single update with index=2 and value=1111 (1011 | 1100 = 1111)"
        );
    }

    #[test]
    fn test_or_gate_3input_4bit() {
        let gate = Gate::new(GateKind::Or, 4, 3);
        let in_a = bitarr![1, 0, 1, 1];
        let in_b = bitarr![1, 1, 0, 0];
        let in_c = bitarr![0, 1, 1, 0];

        let updates = gate.run(RunContext {
            graphs: &Default::default(),
            old_ports: &[bitarr![Z; 4]; 4],
            new_ports: &[in_a, in_b, in_c, bitarr![Z; 4]],
            inner_state: None
        });

        // 1011 | 1100 | 0110 = 1111;
        assert_eq!(
            updates,
            vec![PortUpdate { index: 3, value: bitarr![1, 1, 1, 1] }],
            "Expected a single update with index=3 and value=1111 (1011 | 1100 | 0110 = 1111)"
        );
    }

    #[test]
    fn test_xor_gate() {
        let gate = Gate::new(GateKind::Xor, 1, 2);
        let in_a = bitarr![0];
        let in_b = bitarr![1];

        let updates = gate.run(RunContext {
            graphs: &Default::default(),
            old_ports: &[bitarr![Z]; 3],
            new_ports: &[in_a, in_b, bitarr![Z]],
            inner_state: None
        });

        // 1 ^ 0 = 1;
        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: bitarr![1] }],
            "Expected a single update with index=2 and value=1 (1 ^ 0 = 1)"
        );
    }

    #[test]
    fn test_xor_gate_multi_bit() {
        let gate = Gate::new(GateKind::Xor, 4, 2);
        let in_a = bitarr![1, 0, 1, 1];
        let in_b = bitarr![1, 1, 0, 1];

        let updates = gate.run(RunContext {
            graphs: &Default::default(),
            old_ports: &[bitarr![Z; 4]; 3],
            new_ports: &[in_a, in_b, bitarr![Z; 4]],
            inner_state: None
        });

        // 1011 ^ 1101 = 0110;
        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: bitarr![0, 1, 1, 0] }],
            "Expected a single update with index=2 and value=0110 (1011 ^ 1101 = 0110)"
        );
    }

    #[test]
    fn test_xor_gate_3input_4bit() {
        let gate = Gate::new(GateKind::Xor, 4, 3);
        let in_a = bitarr![1, 0, 1, 1];
        let in_b = bitarr![1, 1, 0, 1];
        let in_c = bitarr![0, 1, 1, 0];

        let updates = gate.run(RunContext {
            graphs: &Default::default(),
            old_ports: &[bitarr![Z; 4]; 4],
            new_ports: &[in_a, in_b, in_c, bitarr![Z; 4]],
            inner_state: None
        });

        // 1011 ^ 1101 ^ 0110 = 0000;
        assert_eq!(
            updates,
            vec![PortUpdate { index: 3, value: bitarr![0, 0, 0, 0] }],
            "Expected a single update with index=3 and value=0000 (1011 ^ 1101 ^ 0110 = 0000)"
        );
    }

    #[test]
    fn test_nand_gate() {
        let gate = Gate::new(GateKind::Nand, 1, 2);
        let in_a = bitarr![0];
        let in_b = bitarr![1];

        let updates = gate.run(RunContext {
            graphs: &Default::default(),
            old_ports: &[bitarr![Z]; 3],
            new_ports: &[in_a, in_b, bitarr![Z]],
            inner_state: None
        });

        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: bitarr![1] }],
            "Expected a single update with index=2 and value=1 (!(1 & 0) = 1)"
        );
    }

    #[test]
    fn test_nand_gate_multi_bit() {
        let gate = Gate::new(GateKind::Nand, 4, 2);
        let in_a = bitarr![1, 0, 1, 1];
        let in_b = bitarr![1, 1, 0, 1];

        let updates = gate.run(RunContext {
            graphs: &Default::default(),
            old_ports: &[bitarr![Z; 4]; 3],
            new_ports: &[in_a, in_b, bitarr![Z; 4]],
            inner_state: None
        });

        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: bitarr![0, 1, 1, 0] }],
            "Expected a single update with index=2 and value=0110 (!(1011 & 1101) = 0110)"
        );
    }

    #[test]
    fn test_nand_gate_3input_4bit() {
        let gate = Gate::new(GateKind::Nand, 4, 3);
        let in_a = bitarr![1, 0, 1, 1];
        let in_b = bitarr![1, 1, 0, 1];
        let in_c = bitarr![1, 1, 1, 0];

        let updates = gate.run(RunContext {
            graphs: &Default::default(),
            old_ports: &[bitarr![Z; 4]; 4],
            new_ports: &[in_a, in_b, in_c, bitarr![Z; 4]],
            inner_state: None
        });

        // !(1011 & 1101 & 1110) = 0111;
        assert_eq!(
            updates,
            vec![PortUpdate { index: 3, value: bitarr![0, 1, 1, 1] }],
            "Expected a single update with index=3 and value=0111 (!(1011 & 1101 & 1110) = 0111)"
        );
    }

    #[test]
    fn test_nor_gate() {
        let gate = Gate::new(GateKind::Nor, 1, 2);
        let in_a = bitarr![0];
        let in_b = bitarr![1];

        let updates = gate.run(RunContext {
            graphs: &Default::default(),
            old_ports: &[bitarr![Z]; 3],
            new_ports: &[in_a, in_b, bitarr![Z]],
            inner_state: None
        });

        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: bitarr![0] }],
            "Expected a single update with index=2 and value=0 (!(1 | 0) = 0)"
        );
    }

    #[test]
    fn test_nor_gate_multi_bit() {
        let gate = Gate::new(GateKind::Nor, 4, 2);
        let in_a = bitarr![1, 0, 1, 1];
        let in_b = bitarr![1, 1, 0, 1];

        let updates = gate.run(RunContext {
            graphs: &Default::default(),
            old_ports: &[bitarr![Z; 4]; 3],
            new_ports: &[in_a, in_b, bitarr![Z; 4]],
            inner_state: None
        });

        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: bitarr![0, 0, 0, 0] }],
            "Expected a single update with index=2 and value=0000 (!(1011 | 1101) = 0000)"
        );
    }

    #[test]
    fn test_nor_gate_3input_4bit() {
        let gate = Gate::new(GateKind::Nor, 4, 3);
        let in_a = bitarr![1, 0, 1, 1];
        let in_b = bitarr![1, 1, 0, 1];
        let in_c = bitarr![0, 1, 1, 0];

        let updates = gate.run(RunContext {
            graphs: &Default::default(),
            old_ports: &[bitarr![Z; 4]; 4],
            new_ports: &[in_a, in_b, in_c, bitarr![Z; 4]],
            inner_state: None
        });

        // !(1011 | 1101 | 0110) = 0000;
        assert_eq!(
            updates,
            vec![PortUpdate { index: 3, value: bitarr![0, 0, 0, 0] }],
            "Expected a single update with index=3 and value=0000 (!(1011 | 1101 | 0110) = 0000)"
        );
    }

    #[test]
    fn test_xnor_gate() {
        let gate = Gate::new(GateKind::Xnor, 1, 2);
        let in_a = bitarr![0];
        let in_b = bitarr![1];

        let updates = gate.run(RunContext {
            graphs: &Default::default(),
            old_ports: &[bitarr![Z]; 3],
            new_ports: &[in_a, in_b, bitarr![Z]],
            inner_state: None
        });

        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: bitarr![0] }],
            "Expected a single update with index=2 and value=0 (!(1 ^ 0) = 0)"
        );
    }

    #[test]
    fn test_xnor_gate_multi_bit() {
        let gate = Gate::new(GateKind::Xnor, 4, 2);
        let in_a = bitarr![1, 0, 1, 1];
        let in_b = bitarr![1, 1, 0, 1];

        let updates = gate.run(RunContext {
            graphs: &Default::default(),
            old_ports: &[bitarr![Z; 4]; 3],
            new_ports: &[in_a, in_b, bitarr![Z; 4]],
            inner_state: None
        });

        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: bitarr![1, 0, 0, 1] }],
            "Expected a single update with index=2 and value=1001 (!(1011 ^ 1101) = 1001)"
        );
    }

    #[test]
    fn test_xnor_gate_3input_4bit() {
        let gate = Gate::new(GateKind::Xnor, 4, 3);
        let in_a = bitarr![1, 0, 1, 1];
        let in_b = bitarr![1, 1, 0, 1];
        let in_c = bitarr![0, 1, 1, 0];

        let updates = gate.run(RunContext {
            graphs: &Default::default(),
            old_ports: &[bitarr![Z; 4]; 4],
            new_ports: &[in_a, in_b, in_c, bitarr![Z; 4]],
            inner_state: None
        });

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

        let updates = gate.run(RunContext {
            graphs: &Default::default(),
            old_ports: &[bitarr![Z]; 2],
            new_ports: &[in_a, bitarr![Z]],
            inner_state: None
        });

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

        let updates = gate.run(RunContext {
            graphs: &Default::default(),
            old_ports: &[bitarr![Z; 4]; 2],
            new_ports: &[in_a, bitarr![Z; 4]],
            inner_state: None
        });

        assert_eq!(
            updates,
            vec![PortUpdate { index: 1, value: bitarr![0, 1, 0, 0] }],
            "Expected a single update with index=1 and value=0100 (!1011 = 0100)"
        );
    }

    #[test]
    fn test_tristate_off() {
        let gate = TriState::new(1);
        let in_a = bitarr![0];

        let updates = gate.run(RunContext {
            graphs: &Default::default(),
            old_ports: &[bitarr![Z]; 3],
            new_ports: &[bitarr![0], in_a, bitarr![Z]],
            inner_state: None
        });

        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: bitarr![Z] }]
        );
    }

    #[test]
    fn test_tristate_on() {
        let gate = TriState::new(1);
        let in_a = bitarr![0];

        let updates = gate.run(RunContext {
            graphs: &Default::default(),
            old_ports: &[bitarr![Z]; 3],
            new_ports: &[bitarr![1], in_a, bitarr![Z]],
            inner_state: None
        });

        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: in_a }],
        );
    }

    #[test]
    fn test_tristate_multi_bit_off() {
        let gate = TriState::new(4);
        let in_a = bitarr![1, 0, 1, 1];

        let updates = gate.run(RunContext {
            graphs: &Default::default(),
            old_ports: &[bitarr![Z], bitarr![Z; 4], bitarr![Z; 4]],
            new_ports: &[bitarr![0], in_a, bitarr![Z; 4]],
            inner_state: None
        });

        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: bitarr![Z; 4] }]
        );
    }

    #[test]
    fn test_tristate_multi_bit_on() {
        let gate = TriState::new(4);
        let in_a = bitarr![1, 0, 1, 1];

        let updates = gate.run(RunContext {
            graphs: &Default::default(),
            old_ports: &[bitarr![Z], bitarr![Z; 4], bitarr![Z; 4]],
            new_ports: &[bitarr![1], in_a, bitarr![Z; 4]],
            inner_state: None
        });

        assert_eq!(
            updates,
            vec![PortUpdate { index: 2, value: in_a }]
        );
    }

    mod input_validation {
        use super::*;

        #[test]
        #[should_panic]
        fn input_validate_and() {
            let gate = Gate::new(GateKind::And, 4, 2);
            // Should fail input validation
            let bad_in = bitarr![1, 1, 1];
            let good_in = bitarr![1, 0, 1, 0];
            let _ = gate.run(RunContext {
                graphs: &Default::default(),
                old_ports: &[bitarr![Z; 4]; 3],
                new_ports: &[bad_in, good_in, bitarr![Z; 4]],
                inner_state: None
            });
        } 

        #[test]
        #[should_panic]
        fn input_validate_or() {
            let gate = Gate::new(GateKind::Or, 4, 2);
            // Should fail input validation
            let bad_in = bitarr![1, 1, 1];
            let good_in = bitarr![1, 0, 1, 0];
            let _ = gate.run(RunContext {
                graphs: &Default::default(),
                old_ports: &[bitarr![Z; 4]; 3],
                new_ports: &[bad_in, good_in, bitarr![Z; 4]],
                inner_state: None
            });
        } 

        #[test]
        #[should_panic]
        fn input_validate_xor() {
            let gate = Gate::new(GateKind::Xor, 4, 2);
            // Should fail input validation
            let bad_in = bitarr![1, 1, 1];
            let good_in = bitarr![1, 0, 1, 0];
            let _ = gate.run(RunContext {
                graphs: &Default::default(),
                old_ports: &[bitarr![Z; 4]; 3],
                new_ports: &[bad_in, good_in, bitarr![Z; 4]],
                inner_state: None
            });
        } 

        #[test]
        #[should_panic]
        fn input_validate_nand() {
            let gate = Gate::new(GateKind::Nand, 4, 2);
            // Should fail input validation
            let bad_in = bitarr![1, 1, 1];
            let good_in = bitarr![1, 0, 1, 0];
            let _ = gate.run(RunContext {
                graphs: &Default::default(),
                old_ports: &[bitarr![Z; 4]; 3],
                new_ports: &[bad_in, good_in, bitarr![Z; 4]],
                inner_state: None
            });
        } 

        #[test]
        #[should_panic]
        fn input_validate_nor() {
            let gate = Gate::new(GateKind::Nor, 4, 2);
            // Should fail input validation
            let bad_in = bitarr![1, 1, 1];
            let good_in = bitarr![1, 0, 1, 0];
            let _ = gate.run(RunContext {
                graphs: &Default::default(),
                old_ports: &[bitarr![Z; 4]; 3],
                new_ports: &[bad_in, good_in, bitarr![Z; 4]],
                inner_state: None
            });
        } 

        #[test]
        #[should_panic]
        fn input_validate_xnor() {
            let gate = Gate::new(GateKind::Xnor, 4, 2);
            // Should fail input validation
            let bad_in = bitarr![1, 1, 1];
            let good_in = bitarr![1, 0, 1, 0];
            let _ = gate.run(RunContext {
                graphs: &Default::default(),
                old_ports: &[bitarr![Z; 4]; 3],
                new_ports: &[bad_in, good_in, bitarr![Z; 4]],
                inner_state: None
            });
        } 

        #[test]
        #[should_panic]
        fn input_validate_not() {
            let gate = Not::new(4);
            // Should fail input validation
            let bad_in = bitarr![1, 1, 1];
            let _ = gate.run(RunContext {
                graphs: &Default::default(),
                old_ports: &[bitarr![Z; 4]; 2],
                new_ports: &[bad_in, bitarr![Z; 4]],
                inner_state: None
            });
        } 

        #[test]
        #[should_panic]
        fn input_validate_tristate() {
            let gate = TriState::new(4);
            // Should fail input validation
            let bad_in = bitarr![1, 1, 1];
            let _ = gate.run(RunContext {
                graphs: &Default::default(),
                old_ports: &[bitarr![Z], bitarr![Z; 4], bitarr![Z; 4]],
                new_ports: &[bitarr![Z], bad_in, bitarr![Z; 4]],
                inner_state: None
            });
        } 
    }
}
