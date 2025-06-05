use crate::bitarray::{BitArray, BitState};

pub struct PortTrigger {
    pub port: usize,
    pub value: BitArray
}
impl PortTrigger {
    pub fn new(port: usize, value: BitArray) -> Self {
        Self { port, value }
    }
    fn seq(n: impl IntoIterator<Item=BitArray>) -> Vec<Self> {
        Self::seq_opt(n.into_iter().map(Some))
    }
    fn seq_opt(n: impl IntoIterator<Item=Option<BitArray>>) -> Vec<Self> {
        n.into_iter()
            .enumerate()
            .filter_map(|(port, v)| Some(PortTrigger { port, value: v? }))
            .collect()
    }
}
pub trait Component {
    fn inputs(&self) -> Vec<u8>;
    fn outputs(&self) -> Vec<u8>;
    #[must_use]
    fn run(&mut self, inp: &[BitArray]) -> Vec<PortTrigger>;
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
pub enum Node {
    Value(BitArray),
    Function(NodeFnType)
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

    fn run(&mut self, inp: &[BitArray]) -> Vec<PortTrigger> {
        fn reduce(inp: &[BitArray], bitsize: u8, f: impl FnMut(BitArray, BitArray) -> BitArray) -> BitArray {
            inp.iter()
                .cloned()
                .reduce(f)
                .unwrap_or_else(|| BitArray::repeat(BitState::Unk, bitsize))
        }
        fn one(t: BitArray) -> Vec<PortTrigger> {
            PortTrigger::seq([t])
        }
        // TODO: bitsize

        match self {
            NodeFnType::Transistor => todo!(),
            NodeFnType::Splitter => todo!(),

            NodeFnType::And  => one(reduce(inp, 64, |a, b| a & b)),
            NodeFnType::Or   => one(reduce(inp, 64, |a, b| a | b)),
            NodeFnType::Xor  => one(reduce(inp, 64, |a, b| a ^ b)),
            NodeFnType::Nand => one(reduce(inp, 64, |a, b| !(a & b))),
            NodeFnType::Nor  => one(reduce(inp, 64, |a, b| !(a | b))),
            NodeFnType::Xnor => one(reduce(inp, 64, |a, b| !(a ^ b))),
            NodeFnType::Not  => one(!inp[0].clone()),
            NodeFnType::TriState => {
                let gate = inp[0].index(0);
                let input = inp[1].clone();
                let bitsize = input.len();

                one(match gate {
                    BitState::Low | BitState::Imped => BitArray::repeat(BitState::Imped, bitsize),
                    BitState::High => input,
                    BitState::Unk => BitArray::repeat(BitState::Unk, bitsize),
                })
            },

            NodeFnType::Mux => {
                let m_sel = inp[0].to_u64();
                let bitsize = inp[1].len();
                match m_sel {
                    Ok(sel) => one(inp[sel as usize + 1].clone()),
                    Err(e) => one(BitArray::repeat(e.bit_state(), bitsize)),
                }
            },
            NodeFnType::Decoder => {
                let m_sel = inp[0].to_u64();
                let n_outputs = inp[0].len();

                match m_sel {
                    Ok(sel) => (0..n_outputs)
                        .map(|i| (i, u64::from(i) == sel))
                        .map(|(i, b)| PortTrigger { port: usize::from(i), value: BitArray::repeat(b.into(), 1) })
                        .collect(),
                    Err(e) => (0..n_outputs)
                        .map(|i| PortTrigger { port: usize::from(i), value: BitArray::repeat(e.bit_state(), 1) })
                        .collect(),
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
            fn run(&mut self, inp: &[BitArray]) -> Vec<PortTrigger> {
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
                fn run(&mut self, inp: &[BitArray]) -> Vec<PortTrigger> {
                    let value = inp.iter()
                        .cloned()
                        .reduce($f)
                        .unwrap_or_else(|| BitArray::repeat(BitState::Unk, self.props.bitsize));
    
                    vec![PortTrigger { port: 0, value }]
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

    fn run(&mut self, inp: &[BitArray]) -> Vec<PortTrigger> {
        vec![PortTrigger { port: 0, value: !inp[0].clone() }]
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

    fn run(&mut self, inp: &[BitArray]) -> Vec<PortTrigger> {
        let gate = inp[0].index(0);
        let result = match gate {
            BitState::High => inp[1].clone(),
            BitState::Low | BitState::Imped => BitArray::repeat(BitState::Imped, self.props.bitsize),
            BitState::Unk => BitArray::repeat(BitState::Unk, self.props.bitsize),
        };
        vec![PortTrigger { port: 0, value: result }]
    }
}