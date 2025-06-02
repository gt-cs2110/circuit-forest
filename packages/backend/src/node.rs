use crate::bitarray::BitArray;
use Sensitivity::*;

pub struct PortTrigger {
    pub port: usize,
    pub value: BitArray
}
impl PortTrigger {
    pub fn new(port: usize, value: BitArray) -> Self {
        Self { port, value }
    }
}
pub trait Component {
    fn num_inputs(&self) -> usize;
    fn num_outputs(&self) -> usize;
    #[must_use]
    fn run(&mut self, inp: &[BitArray]) -> Vec<PortTrigger>;
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Sensitivity {
    Anyedge, Posedge, Negedge, DontCare
}
fn on_update<I, F, const N: usize>(inp: &[BitArray], sensitivities: [Sensitivity; N], apply: F) -> Vec<PortTrigger>
    where 
        I: IntoIterator<Item=PortTrigger>,
        F: FnOnce([BitArray; N]) -> I
{
    // TODO: use sensitivities arg
    if true {
        let inp_arr = <&[_; N]>::try_from(inp).unwrap().clone();
        apply(inp_arr).into_iter().collect()
    } else {
        vec![]
    }
}
pub enum Node {
    Value(BitArray),
    Function(NodeFnType)
}
pub enum NodeFnType {
    And, Or, Xor, Nand, Nor, Xnor, Not
}
impl Component for NodeFnType {
    fn num_inputs(&self) -> usize {
        match self {
            NodeFnType::And  => 2,
            NodeFnType::Or   => 2,
            NodeFnType::Xor  => 2,
            NodeFnType::Nand => 2,
            NodeFnType::Nor  => 2,
            NodeFnType::Xnor => 2,
            NodeFnType::Not  => 1,
        }
    }
    fn num_outputs(&self) -> usize {
        match self {
            NodeFnType::And  => 1,
            NodeFnType::Or   => 1,
            NodeFnType::Xor  => 1,
            NodeFnType::Nand => 1,
            NodeFnType::Nor  => 1,
            NodeFnType::Xnor => 1,
            NodeFnType::Not  => 1,
        }
    }

    fn run(&mut self, inp: &[BitArray]) -> Vec<PortTrigger> {
        match self {
            NodeFnType::And  => on_update(inp, [Anyedge; 2], |[a, b]| [PortTrigger::new(0, a & b)]),
            NodeFnType::Or   => on_update(inp, [Anyedge; 2], |[a, b]| [PortTrigger::new(0, a | b)]),
            NodeFnType::Xor  => on_update(inp, [Anyedge; 2], |[a, b]| [PortTrigger::new(0, a ^ b)]),
            NodeFnType::Nand => on_update(inp, [Anyedge; 2], |[a, b]| [PortTrigger::new(0, !(a & b))]),
            NodeFnType::Nor  => on_update(inp, [Anyedge; 2], |[a, b]| [PortTrigger::new(0, !(a | b))]),
            NodeFnType::Xnor => on_update(inp, [Anyedge; 2], |[a, b]| [PortTrigger::new(0, !(a ^ b))]),
            NodeFnType::Not  => on_update(inp, [Anyedge; 1], |[a]|    [PortTrigger::new(0, !a)]),
        }
    }
}
