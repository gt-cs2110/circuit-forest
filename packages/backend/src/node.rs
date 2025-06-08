use crate::bitarray::{BitArray, BitState};

pub trait Component {
    fn inputs(&self) -> Vec<u8>;
    fn outputs(&self) -> Vec<u8>;
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
pub enum NodeFnType {
    Transistor, Splitter,
    And, Or, Xor, Nand, Nor, Xnor, Not, TriState,
    Mux, Decoder
}
impl Component for NodeFnType {
    fn inputs(&self) -> Vec<u8> {
        vec![64; 2]
    }
    fn outputs(&self) -> Vec<u8> {
        vec![64; 1]
    }

    fn run(&mut self, inp: &[BitArray]) -> Vec<BitArray> {
        fn reduce(inp: &[BitArray], bitsize: u8, f: impl FnMut(BitArray, BitArray) -> BitArray) -> BitArray {
            inp.iter()
                .cloned()
                .reduce(f)
                .unwrap_or_else(|| BitArray::unknown(bitsize))
        }
        // TODO: bitsize

        match self {
            NodeFnType::Transistor => todo!(),
            NodeFnType::Splitter => todo!(),

            NodeFnType::And  => vec![reduce(inp, 64, |a, b| a & b)],
            NodeFnType::Or   => vec![reduce(inp, 64, |a, b| a | b)],
            NodeFnType::Xor  => vec![reduce(inp, 64, |a, b| a ^ b)],
            NodeFnType::Nand => vec![reduce(inp, 64, |a, b| !(a & b))],
            NodeFnType::Nor  => vec![reduce(inp, 64, |a, b| !(a | b))],
            NodeFnType::Xnor => vec![reduce(inp, 64, |a, b| !(a ^ b))],
            NodeFnType::Not  => vec![!inp[0].clone()],
            NodeFnType::TriState => {
                let gate = inp[0].index(0);
                let input = inp[1].clone();
                let bitsize = input.len();

                vec![match gate {
                    BitState::Low | BitState::Imped => BitArray::floating(bitsize),
                    BitState::High => input,
                    BitState::Unk => BitArray::unknown(bitsize),
                }]
            },

            NodeFnType::Mux => {
                let m_sel = inp[0].to_u64();
                let bitsize = inp[1].len();
                match m_sel {
                    Ok(sel) => vec![inp[sel as usize + 1].clone()],
                    Err(e) => vec![BitArray::repeat(e.bit_state(), bitsize)],
                }
            },
            NodeFnType::Decoder => {
                let m_sel = inp[0].to_u64();
                let n_outputs = inp[0].len();

                match m_sel {
                    Ok(sel) => (0..n_outputs)
                        .map(|i| BitArray::repeat((u64::from(i) == sel).into(), 1))
                        .collect(),
                    Err(e) => vec![BitArray::repeat(e.bit_state(), 1); usize::from(n_outputs)],
                }
            },
        }
    }
}

macro_rules! decl_component_enum {
    ($ComponentEnum:ident: $($Component:ident),*$(,)?) => {
        pub enum $ComponentEnum {
            $($Component($Component)),*
        }
        impl Component for $ComponentEnum {
            fn inputs(&self) -> Vec<u8> {
                match self {
                    $(
                        Self::$Component(c) => c.inputs(),
                    )*
                }
            }
            fn outputs(&self) -> Vec<u8> {
                match self {
                    $(
                        Self::$Component(c) => c.outputs(),
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
    }
}
decl_component_enum!(NodeFn: And, Or, Xor, Nand, Nor, Xnor, Not, TriState);
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
            impl Component for $Id {
                fn inputs(&self) -> Vec<u8> {
                    vec![self.props.bitsize; usize::from(self.props.n_inputs)]
                }
                fn outputs(&self) -> Vec<u8> {
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
impl Component for Not {
    fn inputs(&self) -> Vec<u8> {
        vec![self.props.bitsize]
    }

    fn outputs(&self) -> Vec<u8> {
        vec![self.props.bitsize]
    }

    fn run(&mut self, inp: &[BitArray]) -> Vec<BitArray> {
        vec![!inp[0].clone()]
    }
}

pub struct TriState {
    props: BufNotProperties
}
impl Component for TriState {
    fn inputs(&self) -> Vec<u8> {
        vec![1, self.props.bitsize]
    }

    fn outputs(&self) -> Vec<u8> {
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