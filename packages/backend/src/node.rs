use crate::bitarray::{BitArray, BitState};

pub struct FunctionNode {
    state: Vec<BitArray>,
    func: ComponentFn
}
impl FunctionNode {
    pub fn new(mut func: ComponentFn) -> Self {
        let mut state: Vec<_> = func.ports()
            .into_iter()
            .map(BitArray::floating)
            .collect();
        func.initialize(&mut state);

        Self { state, func }
    }
}
pub struct PortUpdate {
    pub index: usize,
    pub value: BitArray
}

pub trait Component {
    fn ports(&self) -> Vec<u8>;
    fn initialize(&mut self, _state: &mut [BitArray]) {}
    #[must_use]
    fn run(&mut self, old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate>;
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Sensitivity {
    Anyedge, Posedge, Negedge, DontCare
}
impl Sensitivity {
    pub fn activated(self, old: BitArray, new: BitArray) -> bool {
        assert_eq!(old.len(), new.len(), "Bit length should be the same");
        match self {
            Sensitivity::Anyedge  => old != new,
            Sensitivity::Posedge  => old.all_low() && new.all_high(),
            Sensitivity::Negedge  => old.all_high() && new.all_low(),
            Sensitivity::DontCare => false,
        }
    }
    pub fn any_activated(self, old: &[BitArray], new: &[BitArray]) -> bool {
        assert_eq!(old.len(), new.len(), "Array size should be the same");
        std::iter::zip(old, new)
            .any(|(&o, &n)| self.activated(o, n))
    }
}

macro_rules! decl_component_enum {
    ($ComponentEnum:ident: $($Component:ident),*$(,)?) => {
        pub enum $ComponentEnum {
            $($Component($Component)),*
        }
        impl Component for $ComponentEnum {
            fn ports(&self) -> Vec<u8> {
                match self {
                    $(
                        Self::$Component(c) => c.ports(),
                    )*
                }
            }
            fn initialize(&mut self, state: &mut [BitArray]) {
                match self {
                    $(
                        Self::$Component(c) => c.initialize(state),
                    )*
                }
            }
            fn run(&mut self, old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate> {
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
    Mux, Demux, Decoder, Splitter,
);

pub const MIN_GATE_INPUTS: u8 = 2;
pub const MAX_GATE_INPUTS: u8 = 64;
pub struct GateProperties {
    bitsize: u8,
    n_inputs: u8
}

macro_rules! gates {
    ($($Id:ident: $f:expr),*$(,)?) => {
        $(
            pub struct $Id {
                props: GateProperties
            }
            impl $Id {
                pub fn new(mut bitsize: u8, mut n_inputs: u8) -> Self {
                    bitsize = bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE);
                    n_inputs = n_inputs.clamp(MIN_GATE_INPUTS, MAX_GATE_INPUTS);
                    Self { props: GateProperties { bitsize, n_inputs }}
                }
            }
            impl Component for $Id {
                fn ports(&self) -> Vec<u8> {
                    std::iter::repeat_n(self.props.bitsize, usize::from(self.props.n_inputs)) // inputs
                        .chain([self.props.bitsize]) // outputs
                        .collect()
                }
                fn run(&mut self, old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate> {
                    let value = inp[..usize::from(self.props.n_inputs)].iter()
                        .cloned()
                        .reduce($f)
                        .unwrap_or_else(|| BitArray::unknown(self.props.bitsize));
    
                    vec![PortUpdate { index: usize::from(self.props.n_inputs), value }]
                }
            }
        )*
    }
}

gates! {
    And:  |a, b| a & b,
    Or:   |a, b| a | b,
    Xor:  |a, b| a ^ b,
    Nand: |a, b| !(a & b),
    Nor:  |a, b| !(a | b),
    Xnor: |a, b| !(a ^ b),
}

pub struct BufNotProperties {
    bitsize: u8
}
pub struct Not {
    props: BufNotProperties
}
impl Not {
    pub fn new(mut bitsize: u8) -> Self {
        bitsize = bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE);
        Self { props: BufNotProperties { bitsize }}
    }
}
impl Component for Not {
    fn ports(&self) -> Vec<u8> {
        vec![self.props.bitsize; 2]
    }

    fn run(&mut self, old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate> {
        vec![PortUpdate { index: 1, value: !inp[0] }]
    }
}

pub struct TriState {
    props: BufNotProperties
}
impl TriState {
    pub fn new(mut bitsize: u8) -> Self {
        bitsize = bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE);
        Self { props: BufNotProperties { bitsize }}
    }
}
impl Component for TriState {
    fn ports(&self) -> Vec<u8> {
        // selector, input, output
        vec![1, self.props.bitsize, self.props.bitsize]
    }

    fn run(&mut self, old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate> {
        let gate = inp[0].index(0);
        let result = match gate {
            BitState::High => inp[1],
            BitState::Low | BitState::Imped => BitArray::floating(self.props.bitsize),
            BitState::Unk => BitArray::unknown(self.props.bitsize),
        };
        vec![PortUpdate { index: 2, value: result }]
    }
}

pub const MIN_SELSIZE: u8 = 1;
pub const MAX_SELSIZE: u8 = 8;
pub struct MuxProperties {
    bitsize: u8,
    selsize: u8
}
pub struct Mux {
    props: MuxProperties
}
impl Mux {
    pub fn new(mut bitsize: u8, mut selsize: u8) -> Self {
        bitsize = bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE);
        selsize = selsize.clamp(MIN_SELSIZE, MAX_SELSIZE);
        Self { props: MuxProperties { bitsize, selsize }}
    }
}
impl Component for Mux {
    fn ports(&self) -> Vec<u8> {
        let mut sizes = vec![self.props.selsize]; // selector
        sizes.extend(std::iter::repeat_n(self.props.bitsize, 1 << self.props.selsize)); // inputs
        sizes.push(self.props.bitsize); //output
        sizes
    }

    fn run(&mut self, old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate> {
        let m_sel = u64::try_from(inp[0]);
        let result = match m_sel {
            Ok(sel) => inp[sel as usize + 1],
            Err(e) => BitArray::repeat(e.bit_state(), self.props.bitsize),
        };
        vec![PortUpdate { index: (1 << self.props.selsize) + 1, value: result }]
    }
}
pub struct Demux {
    props: MuxProperties
}
impl Demux {
    pub fn new(mut bitsize: u8, mut selsize: u8) -> Self {
        bitsize = bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE);
        selsize = selsize.clamp(MIN_SELSIZE, MAX_SELSIZE);
        Self { props: MuxProperties { bitsize, selsize }}
    }
}
impl Component for Demux {
    fn ports(&self) -> Vec<u8> {
        let mut sizes = vec![self.props.selsize, self.props.bitsize]; // selector and input
        sizes.extend(std::iter::repeat_n(self.props.bitsize, 1 << self.props.selsize)); // outputs
        sizes
    }
    fn run(&mut self, old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate> {
        let m_sel = u64::try_from(inp[0]);
        let result = match m_sel {
            Ok(sel) => {
                let mut result = vec![BitArray::repeat(BitState::Low, self.props.bitsize); 1 << self.props.selsize];
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

pub struct DecoderProperties {
    selsize: u8
}
pub struct Decoder {
    props: DecoderProperties
}
impl Decoder {
    pub fn new(mut selsize: u8) -> Self {
        selsize = selsize.clamp(MIN_SELSIZE, MAX_SELSIZE);
        Self { props: DecoderProperties { selsize }}
    }
}
impl Component for Decoder {
    fn ports(&self) -> Vec<u8> {
        let mut sizes = vec![self.props.selsize]; // selector
        sizes.extend(std::iter::repeat_n(1, 1 << self.props.selsize)); // outputs
        sizes
    }

    fn run(&mut self, old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate> {
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

pub struct Splitter {
    props: BufNotProperties
}
impl Splitter {
    pub fn new(mut bitsize: u8) -> Self {
        bitsize = bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE);
        Self { props: BufNotProperties { bitsize } }
    }
}
impl Component for Splitter {
    fn ports(&self) -> Vec<u8> {
        let mut sizes = vec![self.props.bitsize]; // left
        sizes.extend(std::iter::repeat_n(1, usize::from(self.props.bitsize)));
        sizes
    }

    fn run(&mut self, old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate> {
        if Sensitivity::Anyedge.activated(old_inp[0], inp[0]) {
            std::iter::zip(1..=usize::from(self.props.bitsize), inp[0])
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