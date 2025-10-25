//! Digital Logic Components and Gates for simulation.
//! 
//! This module defines various digital logic components such as gates (AND, OR, NOT, etc.),
//! multiplexers, demultiplexers, decoders, splitters, and registers as well as the
//! traits and structures needed to represent and simulate their behavior.
//! 
//! ## The node module notably consists of:
//! - **[`Component`]**: An interface for all digital logic components, defining methods for port configuration, initialization, and execution.
//! - **[`PortType`] and [`PortProperties`]**: Enumerations and structures to define the types and properties of ports for components.
//! - **[`PortUpdate`]**: A structure representing updates to port values during simulation.
//! - **Digital Logic Components**: Implementations of basic logic components used to simulate digital circuits.
use crate::bitarray::{BitArray, BitState, bitarr};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug)]

/// The type of ports available for a digital logic component.
pub enum PortType {
    /// A port which accepts incoming signals.
    Input, 
    /// A port which provides outgoing signals.
    Output, 
    /// A port which can accept and provide signals.
    Inout 
}
impl PortType {
    /// Checks if the port type accepts input signals.
    pub fn accepts_input(self) -> bool {
        matches!(self, PortType::Input | PortType::Inout)
    }

    /// Checks if the port type provides outgoing signals.
    pub fn accepts_output(self) -> bool {
        matches!(self, PortType::Output | PortType::Inout)
    }
}
/// The properties of a port for a digital logic component.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug)]
pub struct PortProperties {
    /// Type of the port.
    pub ty: PortType,
    /// Size of the data the port works with in bits.
    pub bitsize: u8
}

/// A struct representing an update to a port's value during simulation.
/// 
/// This struct should only be used to represent the update 
/// of a [`Output`] or [`Inout`] port.
/// 
/// [`Output`]: PortType::Output
/// [`Inout`]: PortType::Inout
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PortUpdate {
    /// Index of the port being updated.
    /// 
    /// For example, this is an index of 0 when referring
    /// to the first port in a function's port list.
    pub index: usize,
    /// The new value to be assigned to the port at the given index.
    pub value: BitArray
}

/// The interface defining how a digital logic component operates.
pub trait Component {
    /// Returns the vector holding the properties of all ports associated with the component.
    /// 
    /// This is called only once during initialization.
    /// It is assumed that the result of this function will not change when called multiple times.
    fn ports(&self) -> Vec<PortProperties>;
    
    /// Initializes the state (e.g., internal state and port state) of the component.
    /// 
    /// If not specified, by default, the initial port state is set to all floating.
    fn initialize(&self, _state: &mut [BitArray]) {}
    
    /// "Runs" the component's function on a set of inputs, outputting a vector of updated ports
    /// after the function is applied.
    /// 
    /// This function is called after an update is propagated to this component.
    /// When that occurs, this function is called with the original state and updated state
    /// of this component's ports.
    /// 
    /// This function may also panic if `old_inp` and `inp` do not match the port properties
    /// specified by [`Component::ports`].
    #[must_use]
    fn run(&self, old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate>;
}


/// The triggering conditions for components based on a signal change.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Sensitivity {
    /// Triggered on any change in a signal (rising or falling edge).
    Anyedge, 
    /// Triggered on rising edge of a signal (low to high clock transition).
    Posedge, 
    /// Triggered on falling edge of a signal (high to low clock transition).
    Negedge, 
    /// Does not update in response to a signal update.
    DontCare
}
impl Sensitivity {
    /// Checks whether the change between the old and new value
    /// would create a trigger based on this sensitivity.
    /// 
    /// ```
    /// use circuitsim_engine::bitarray::bitarr;
    /// use circuitsim_engine::node::Sensitivity;
    /// 
    /// let lo = bitarr![0];
    /// let hi = bitarr![1];
    /// assert!(Sensitivity::Posedge.activated(lo, hi));
    /// assert!(Sensitivity::Negedge.activated(hi, lo));
    /// ```
    pub fn activated(self, old: BitArray, new: BitArray) -> bool {
        assert_eq!(old.len(), new.len(), "Bit length should be the same");
        match self {
            Sensitivity::Anyedge  => old != new,
            Sensitivity::Posedge  => old.all(BitState::Low) && new.all(BitState::High),
            Sensitivity::Negedge  => old.all(BitState::High) && new.all(BitState::Low),
            Sensitivity::DontCare => false,
        }
    }

    /// Checks whether the changes in values (as specified by the `old` and `new` slices)
    /// would create a trigger based on this sensitivity.
    /// 
    /// ```
    /// use circuitsim_engine::bitarray::bitarr;
    /// use circuitsim_engine::node::Sensitivity;
    /// 
    /// let lo = bitarr![0];
    /// let hi = bitarr![1];
    /// assert!(Sensitivity::Posedge.any_activated(&[lo, lo, lo, lo], &[lo, hi, lo, lo]));
    /// ```
    /// 
    /// This function panics if `old.len() != new.len()`.
    pub fn any_activated(self, old: &[BitArray], new: &[BitArray]) -> bool {
        assert_eq!(old.len(), new.len(), "Array size should be the same");
        std::iter::zip(old, new)
            .any(|(&o, &n)| self.activated(o, n))
    }
}

/// Helper function to more easily define port lists (for [`Component::ports`]).
fn port_list(config: &[(PortProperties, u8)]) -> Vec<PortProperties> {
    config.iter()
        .flat_map(|&(props, ct)| std::iter::repeat_n(props, usize::from(ct)))
        .collect()
}
macro_rules! decl_component_enum {
    ($ComponentEnum:ident: $($Component:ident),*$(,)?) => {
        /// An enum that represents all supported digital logic components.
        pub enum $ComponentEnum {
            $(
                #[allow(missing_docs)]
                $Component($Component)
            ),*
        }
        impl Component for $ComponentEnum {
            fn ports(&self) -> Vec<PortProperties> {
                match self {
                    $(
                        Self::$Component(c) => c.ports(),
                    )*
                }
            }
            fn initialize(&self, state: &mut [BitArray]) {
                match self {
                    $(
                        Self::$Component(c) => c.initialize(state),
                    )*
                }
            }
            fn run(&self, old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate> {
                match self {
                    $(
                        Self::$Component(c) => c.run(old_inp, inp),
                    )*
                }
            }
        }
        $(
            impl From<$Component> for $ComponentEnum {
                fn from(value: $Component) -> Self {
                    Self::$Component(value)
                }
            }
        )*
    }
}
decl_component_enum!(ComponentFn: 
    And, Or, Xor, Nand, Nor, Xnor, Not, TriState, 
    Mux, Demux, Decoder, Splitter, Register
);
/// Minimum number of inputs for multi-input logic gates.
pub const MIN_GATE_INPUTS: u8 = 2;
/// Maximum number of inputs for multi-input logic gates.
pub const MAX_GATE_INPUTS: u8 = 64;


/// The properties for multi-input logic gates.
pub struct GateProperties {
    /// The size of the data the gate works with in bits.
    pub bitsize: u8,
    /// The number of input ports the gate has.
    pub n_inputs: u8
}

macro_rules! gates {
    ($($(#[$m:meta])? $Id:ident: $f:expr, $invert:literal),*$(,)?) => {
        $(
            $(#[$m])?
            pub struct $Id {
                props: GateProperties
            }
            impl $Id {
                /// Creates a new instance of the gate with specified bitsize and number of inputs.
                pub fn new(mut bitsize: u8, mut n_inputs: u8) -> Self {
                    bitsize = bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE);
                    n_inputs = n_inputs.clamp(MIN_GATE_INPUTS, MAX_GATE_INPUTS);
                    Self { props: GateProperties { bitsize, n_inputs }}
                }
            }
            impl Component for $Id {
                fn ports(&self) -> Vec<PortProperties> {
                    port_list(&[
                        // inputs
                        (PortProperties { ty: PortType::Input, bitsize: self.props.bitsize }, self.props.n_inputs),
                        // outputs
                        (PortProperties { ty: PortType::Output, bitsize: self.props.bitsize }, 1),
                    ])
                }
                fn run(&self, _old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate> {
                    let value = inp[..usize::from(self.props.n_inputs)].iter()
                        .cloned()
                        .reduce($f)
                        .unwrap_or_else(|| bitarr![X; self.props.bitsize]);
    
                    vec![PortUpdate {
                        index: usize::from(self.props.n_inputs),
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

/// A structure that holds properties for buffer and NOT gates.
pub struct BufNotProperties {
    /// The size of the data the gate works with in bits.
    pub bitsize: u8
}

/// A NOT gate component.
pub struct Not {
    props: BufNotProperties
}
impl Not {
    /// Creates a new instance of the NOT gate with specified bitsize.
    pub fn new(mut bitsize: u8) -> Self {
        bitsize = bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE);
        Self { props: BufNotProperties { bitsize }}
    }
}
impl Component for Not {
    fn ports(&self) -> Vec<PortProperties> {
        port_list(&[
            // input
            (PortProperties { ty: PortType::Input, bitsize: self.props.bitsize }, 1),
            // output
            (PortProperties { ty: PortType::Output, bitsize: self.props.bitsize }, 1),
        ])
    }

    fn run(&self, _old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate> {
        vec![PortUpdate { index: 1, value: !inp[0] }]
    }
}

/// A tri-state buffer component.
pub struct TriState {
    props: BufNotProperties
}
impl TriState {
    /// Creates a new instance of the tri-state buffer with specified bitsize.
    pub fn new(mut bitsize: u8) -> Self {
        bitsize = bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE);
        Self { props: BufNotProperties { bitsize }}
    }
}
impl Component for TriState {
    fn ports(&self) -> Vec<PortProperties> {
        port_list(&[
            // selector
            (PortProperties { ty: PortType::Input, bitsize: 1 }, 1),
            // input
            (PortProperties { ty: PortType::Input, bitsize: self.props.bitsize }, 1),
            // output
            (PortProperties { ty: PortType::Output, bitsize: self.props.bitsize }, 1),
        ])
    }

    fn run(&self, _old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate> {
        let gate = inp[0].index(0);
        let result = match gate {
            BitState::High => inp[1],
            BitState::Low | BitState::Imped => bitarr![Z; self.props.bitsize],
            BitState::Unk => bitarr![X; self.props.bitsize],
        };
        vec![PortUpdate { index: 2, value: result }]
    }
}
/// Minimum number of selector bits for Mux/Demux/Decoder.
pub const MIN_SELSIZE: u8 = 1;
/// Maximum number of selector bits for Mux/Demux/Decoder.
pub const MAX_SELSIZE: u8 = 8;

/// A structure that holds properties for Mux and Demux components.
pub struct MuxProperties {
    /// The size of the data the component works with
    pub bitsize: u8,
    /// The number of selector bits for component
    pub selsize: u8
}

/// A multiplexer (mux) component.
pub struct Mux {
    props: MuxProperties
}
impl Mux {
    /// Creates a new instance of the Mux with specified bitsize and selector size.
    pub fn new(mut bitsize: u8, mut selsize: u8) -> Self {
        bitsize = bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE);
        selsize = selsize.clamp(MIN_SELSIZE, MAX_SELSIZE);
        Self { props: MuxProperties { bitsize, selsize }}
    }
}
impl Component for Mux {
    fn ports(&self) -> Vec<PortProperties> {
        port_list(&[
            // selector
            (PortProperties { ty: PortType::Input, bitsize: self.props.selsize }, 1),
            // inputs
            (PortProperties { ty: PortType::Input, bitsize: self.props.bitsize }, 1 << self.props.selsize),
            // output
            (PortProperties { ty: PortType::Output, bitsize: self.props.bitsize }, 1),
        ])
    }

    fn run(&self, _old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate> {
        let m_sel = u64::try_from(inp[0]);
        let result = match m_sel {
            Ok(sel) => inp[sel as usize + 1],
            Err(e) => BitArray::repeat(e.bit_state(), self.props.bitsize),
        };
        vec![PortUpdate { index: (1 << self.props.selsize) + 1, value: result }]
    }
}

/// A demultiplexer (demux) component.
pub struct Demux {
    props: MuxProperties
}
impl Demux {
    /// Creates a new instance of the Demux with specified bitsize and selector size.
    pub fn new(mut bitsize: u8, mut selsize: u8) -> Self {
        bitsize = bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE);
        selsize = selsize.clamp(MIN_SELSIZE, MAX_SELSIZE);
        Self { props: MuxProperties { bitsize, selsize }}
    }
}
impl Component for Demux {
    fn ports(&self) -> Vec<PortProperties> {
            port_list(&[
            // selector
            (PortProperties { ty: PortType::Input, bitsize: self.props.selsize }, 1),
            // input
            (PortProperties { ty: PortType::Input, bitsize: self.props.bitsize }, 1),
            // outputs
            (PortProperties { ty: PortType::Output, bitsize: self.props.bitsize }, 1 << self.props.selsize),
        ])
    }
    fn run(&self, _old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate> {
        let m_sel = u64::try_from(inp[0]);
        let result = match m_sel {
            Ok(sel) => {
                let mut result = vec![bitarr![0; self.props.bitsize]; 1 << self.props.selsize];
                result[sel as usize] = inp[1];
                result
            },
            Err(e) => vec![BitArray::repeat(e.bit_state(), self.props.bitsize); 1 << self.props.selsize],
        };

        result.into_iter()
            .enumerate()
            .map(|(i, value)| PortUpdate { index: 2 + i, value })
            .collect()
    }
}

/// A structure that holds properties for decoder components.
pub struct DecoderProperties {
    selsize: u8
}

/// A decoder component.
pub struct Decoder {
    props: DecoderProperties
}
impl Decoder {
    /// Creates a new instance of the Decoder with specified selector size.
    pub fn new(mut selsize: u8) -> Self {
        selsize = selsize.clamp(MIN_SELSIZE, MAX_SELSIZE);
        Self { props: DecoderProperties { selsize }}
    }
}
impl Component for Decoder {
    fn ports(&self) -> Vec<PortProperties> {
        port_list(&[
            // selector
            (PortProperties { ty: PortType::Input, bitsize: self.props.selsize }, 1),
            // outputs
            (PortProperties { ty: PortType::Output, bitsize: 1 }, 1 << self.props.selsize),
        ])
    }

    fn run(&self, _old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate> {
        let m_sel = u64::try_from(inp[0]);
        let result = match m_sel {
            Ok(sel) => {
                let mut result = vec![BitArray::from_iter([BitState::Low]); 1 << self.props.selsize];
                result[sel as usize] = BitArray::from_iter([BitState::High]);
                result
            },
            Err(e) => vec![BitArray::from_iter([e.bit_state()]); 1 << self.props.selsize],
        };

        result.into_iter()
            .enumerate()
            .map(|(i, value)| PortUpdate { index: 1 + i, value })
            .collect()
    }
}

/// A splitter component.
pub struct Splitter {
    props: BufNotProperties
}
impl Splitter {
    /// Creates a new instance of the Splitter with specified bitsize.
    pub fn new(mut bitsize: u8) -> Self {
        bitsize = bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE);
        Self { props: BufNotProperties { bitsize } }
    }
}
impl Component for Splitter {
    fn ports(&self) -> Vec<PortProperties> {
        port_list(&[
            // joined
            (PortProperties { ty: PortType::Inout, bitsize: self.props.bitsize }, 1),
            // split
            (PortProperties { ty: PortType::Inout, bitsize: 1 }, self.props.bitsize),
        ])
    }

    fn run(&self, old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate> {
        if Sensitivity::Anyedge.activated(old_inp[0], inp[0]) {
            std::iter::zip(1.., inp[0])
                .map(|(index, bit)| PortUpdate { index, value: BitArray::from_iter([bit]) })
                .collect()
        } else if Sensitivity::Anyedge.any_activated(&old_inp[1..], &inp[1..]) {
            let value = inp[1..].iter()
                .map(|b| b.index(0))
                .collect();
            vec![PortUpdate { index: 0, value }]
        } else {
            vec![]
        }
    }
}

/// A register component.
pub struct Register {
    props: BufNotProperties
}
impl Register {
    /// Creates a new instance of the Register with specified bitsize.
    pub fn new(mut bitsize: u8) -> Self {
        bitsize = bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE);
        Self { props: BufNotProperties { bitsize } }
    }
}
impl Component for Register {
    fn ports(&self) -> Vec<PortProperties> {
        port_list(&[
            // din
            (PortProperties { ty: PortType::Input, bitsize: self.props.bitsize }, 1),
            // enable, clock, clear
            (PortProperties { ty: PortType::Input, bitsize: 1 }, 3),
            // dout
            (PortProperties { ty: PortType::Output, bitsize: self.props.bitsize }, 1),
        ])
    }
    fn initialize(&self, state: &mut [BitArray]) {
        state[4] = bitarr![0; self.props.bitsize];
    }
    fn run(&self, old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate> {
        if inp[3].all(BitState::High) {
            vec![PortUpdate { index: 4, value: bitarr![0; self.props.bitsize] }]
        } else if Sensitivity::Posedge.activated(old_inp[2], inp[2]) && inp[1].all(BitState::High) {
            vec![PortUpdate { index: 4, value: inp[0] }]
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod gates {
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
    }



}

