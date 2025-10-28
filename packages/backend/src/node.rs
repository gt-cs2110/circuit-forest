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
    fn run(&self, old_ports: &[BitArray], new_ports: &[BitArray]) -> Vec<PortUpdate>{
        self.validate_ports(old_ports);
        self.validate_ports(new_ports);
        self.run_inner(old_ports, new_ports)
    }

    /// Inner run function that, given a set of inputs, applies its modifications to output a vector
    /// of updated ports. This function is wrapped by run to ensure input validation
    fn run_inner(&self, old_ports: &[BitArray], new_ports: &[BitArray]) -> Vec<PortUpdate>;

    /// Validates inputs to ensure all ports match port bitsize.
    fn validate_ports(&self, ports: &[BitArray]) {
        // Only run in debug mode
        if cfg!(debug_assertions) {
            let port_props = self.ports();
            for (i, (bit_vec, port)) in ports.iter().zip(port_props).enumerate() {
                debug_assert_eq!(
                    bit_vec.len(),
                    port.bitsize,
                    "Port {i} has incorrect bit width"
                );
            }
        }
    }
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
            fn run_inner(&self, old_ports: &[BitArray], new_ports: &[BitArray]) -> Vec<PortUpdate> {
                match self {
                    $(
                        Self::$Component(c) => c.run(old_ports, new_ports),
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
    And, Or, Xor, Nand, Nor, Xnor, Not, TriState, Input, Output, Constant,
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
                fn run_inner(&self, _old_ports: &[BitArray], new_ports: &[BitArray]) -> Vec<PortUpdate> {
                    let value = new_ports[..usize::from(self.props.n_inputs)].iter()
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

    fn run_inner(&self, _old_ports: &[BitArray], new_ports: &[BitArray]) -> Vec<PortUpdate> {
        vec![PortUpdate { index: 1, value: !new_ports[0] }]
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

    fn run_inner(&self, _old_ports: &[BitArray], new_ports: &[BitArray]) -> Vec<PortUpdate> {
        let gate = new_ports[0].index(0);
        let result = match gate {
            BitState::High => new_ports[1],
            BitState::Low | BitState::Imped => bitarr![Z; self.props.bitsize],
            BitState::Unk => bitarr![X; self.props.bitsize],
        };
        vec![PortUpdate { index: 2, value: result }]
    }
}

/// An input.
pub struct Input {
    props: BufNotProperties
}
impl Input {
    /// Creates a new instance of the tri-state buffer with specified bitsize.
    pub fn new(mut bitsize: u8) -> Self {
        bitsize = bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE);
        Self { props: BufNotProperties { bitsize } }
    }
}
impl Component for Input {
    fn ports(&self) -> Vec<PortProperties> {
        port_list(&[
            // output
            (PortProperties { ty: PortType::Output, bitsize: self.props.bitsize }, 1),
        ])
    }

    fn run_inner(&self, _old_inp: &[BitArray], _inp: &[BitArray]) -> Vec<PortUpdate> {
        vec![]
    }
}

/// An input.
pub struct Output {
    props: BufNotProperties
}
impl Output {
    /// Creates a new instance of the tri-state buffer with specified bitsize.
    pub fn new(mut bitsize: u8) -> Self {
        bitsize = bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE);
        Self { props: BufNotProperties { bitsize } }
    }
}
impl Component for Output {
    fn ports(&self) -> Vec<PortProperties> {
        port_list(&[
            // output
            (PortProperties { ty: PortType::Input, bitsize: self.props.bitsize }, 1),
        ])
    }

    fn run_inner(&self, _old_inp: &[BitArray], _inp: &[BitArray]) -> Vec<PortUpdate> {
        vec![]
    }
}

/// An input.
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
    fn ports(&self) -> Vec<PortProperties> {
        port_list(&[
            // output
            (PortProperties { ty: PortType::Output, bitsize: self.value.len() }, 1),
        ])
    }

    fn initialize(&self, state: &mut [BitArray]) {
        state[0] = self.value;
    }
    fn run_inner(&self, _old_inp: &[BitArray], _inp: &[BitArray]) -> Vec<PortUpdate> {
        vec![]
    }
}

/// Minimum number of selector bits for Mux/Demux/Decoder.
pub const MIN_SELSIZE: u8 = 1;
/// Maximum number of selector bits for Mux/Demux/Decoder.
pub const MAX_SELSIZE: u8 = 6;

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

    fn run_inner(&self, _old_ports: &[BitArray], new_ports: &[BitArray]) -> Vec<PortUpdate> {
        let m_sel = u64::try_from(new_ports[0]);
        let result = match m_sel {
            Ok(sel) => new_ports[sel as usize + 1],
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
    fn run_inner(&self, _old_ports: &[BitArray], new_ports: &[BitArray]) -> Vec<PortUpdate> {
        let m_sel = u64::try_from(new_ports[0]);
        let result = match m_sel {
            Ok(sel) => {
                let mut result = vec![bitarr![0; self.props.bitsize]; 1 << self.props.selsize];
                result[sel as usize] = new_ports[1];
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

    fn run_inner(&self, _old_ports: &[BitArray], new_ports: &[BitArray]) -> Vec<PortUpdate> {
        let m_sel = u64::try_from(new_ports[0]);
        let result = match m_sel {
            Ok(sel) => {
                let mut result = vec![bitarr![0]; 1 << self.props.selsize];
                result[sel as usize] = bitarr![1];
                result
            },
            Err(e) => vec![BitArray::from(e.bit_state()); 1 << self.props.selsize],
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

    fn run_inner(&self, old_ports: &[BitArray], new_ports: &[BitArray]) -> Vec<PortUpdate> {
        if Sensitivity::Anyedge.activated(old_ports[0], new_ports[0]) {
            std::iter::zip(1.., new_ports[0])
                .map(|(index, bit)| PortUpdate { index, value: BitArray::from(bit) })
                .collect()
        } else if Sensitivity::Anyedge.any_activated(&old_ports[1..], &new_ports[1..]) {
            let value = new_ports[1..].iter()
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
    fn run_inner(&self, old_ports: &[BitArray], new_ports: &[BitArray]) -> Vec<PortUpdate> {
        if new_ports[3].all(BitState::High) {
            vec![PortUpdate { index: 4, value: bitarr![0; self.props.bitsize] }]
        } else if Sensitivity::Posedge.activated(old_ports[2], new_ports[2]) && new_ports[1].all(BitState::High) {
            vec![PortUpdate { index: 4, value: new_ports[0] }]
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn floating_ports(properties: &[PortProperties]) -> Vec<BitArray> {
        properties.iter()
            .map(|p| bitarr![Z; p.bitsize])
            .collect()
    }

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

    mod muxes {
        use super::*;
        use crate::bitarray::BitArray;

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

    mod splitter {
        use super::*;
        use crate::bitarray::BitArray;

        #[test]
        fn test_splitter_split() {
            for bitsize in BitArray::MIN_BITSIZE..=BitArray::MAX_BITSIZE {
                let splitter = Splitter::new(bitsize);
                let props = splitter.ports();

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
                
                let actual = splitter.run(&old_ports, &new_ports);
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
                let props = splitter.ports();

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
                
                let actual = splitter.run(&old_ports, &new_ports);
                let expected = vec![PortUpdate { index: 0, value: joined }];

                assert_eq!(
                    actual,
                    expected,
                    "Splitter should correctly split the input bits"
                );
            }
        }
    }
}
