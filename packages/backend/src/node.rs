use crate::bitarray::{BitArray, BitState};

pub trait Component {
    fn input_sizes(&self) -> Vec<u8>;
    fn output_sizes(&self) -> Vec<u8>;
    #[must_use]
    fn run(&mut self, inp: &[BitArray]) -> Vec<BitArray>;
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Sensitivity {
    Anyedge, Posedge, Negedge, DontCare
}
impl Sensitivity {
    pub fn activated(self, old: &BitArray, new: &BitArray) -> bool {
        assert_eq!(old.len(), new.len(), "Bit length should be the same");
        match self {
            Sensitivity::Anyedge  => old != new,
            Sensitivity::Posedge  => old.all_low() && new.all_high(),
            Sensitivity::Negedge  => old.all_high() && new.all_low(),
            Sensitivity::DontCare => false,
        }
    }
}
fn any_activated(old: &[BitArray], new: &[BitArray], sensitivities: &[Sensitivity]) -> bool {
    assert_eq!(old.len(), new.len(), "Array size should be the same");
    assert_eq!(old.len(), sensitivities.len(), "Array size should be the same");
    old.iter()
        .zip(new)
        .zip(sensitivities)
        .any(|((o, n), s)| s.activated(o, n))
}

macro_rules! decl_component_enum {
    ($ComponentEnum:ident: $($Component:ident),*$(,)?) => {
        pub enum $ComponentEnum {
            $($Component($Component)),*
        }
        impl Component for $ComponentEnum {
            fn input_sizes(&self) -> Vec<u8> {
                match self {
                    $(
                        Self::$Component(c) => c.input_sizes(),
                    )*
                }
            }
            fn output_sizes(&self) -> Vec<u8> {
                match self {
                    $(
                        Self::$Component(c) => c.output_sizes(),
                    )*
                }
            }
            fn run(&mut self, inp: &[BitArray]) -> Vec<BitArray> {
                match self {
                    $(
                        Self::$Component(c) => c.run(inp),
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
decl_component_enum!(ComponentFn: And, Or, Xor, Nand, Nor, Xnor, Not, TriState);

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
                fn input_sizes(&self) -> Vec<u8> {
                    vec![self.props.bitsize; usize::from(self.props.n_inputs)]
                }
                fn output_sizes(&self) -> Vec<u8> {
                    vec![self.props.bitsize]
                }
                fn run(&mut self, inp: &[BitArray]) -> Vec<BitArray> {
                    let value = inp.iter()
                        .cloned()
                        .reduce($f)
                        .unwrap_or_else(|| BitArray::unknown(self.props.bitsize));
    
                    vec![value]
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
    fn input_sizes(&self) -> Vec<u8> {
        vec![self.props.bitsize]
    }

    fn output_sizes(&self) -> Vec<u8> {
        vec![self.props.bitsize]
    }

    fn run(&mut self, inp: &[BitArray]) -> Vec<BitArray> {
        vec![!inp[0].clone()]
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
    fn input_sizes(&self) -> Vec<u8> {
        vec![1, self.props.bitsize]
    }

    fn output_sizes(&self) -> Vec<u8> {
        vec![self.props.bitsize]
    }

    fn run(&mut self, inp: &[BitArray]) -> Vec<BitArray> {
        let gate = inp[0].index(0);
        let result = match gate {
            BitState::High => inp[1].clone(),
            BitState::Low | BitState::Imped => BitArray::floating(self.props.bitsize),
            BitState::Unk => BitArray::unknown(self.props.bitsize),
        };
        vec![result]
    }
}