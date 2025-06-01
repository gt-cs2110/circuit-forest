use crate::bitarray::BitArray;

pub trait Component {
    fn num_inputs(&self) -> usize;
    fn num_outputs(&self) -> usize;
    fn run(&mut self, inp: &[BitArray]) -> Vec<BitArray>;
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

    fn run(&mut self, inp: &[BitArray]) -> Vec<BitArray> {
        match self {
            NodeFnType::And  => vec![inp[0].clone() & inp[1].clone()],
            NodeFnType::Or   => vec![inp[0].clone() | inp[1].clone()],
            NodeFnType::Xor  => vec![inp[0].clone() ^ inp[1].clone()],
            NodeFnType::Nand => vec![!(inp[0].clone() & inp[1].clone())],
            NodeFnType::Nor  => vec![!(inp[0].clone() | inp[1].clone())],
            NodeFnType::Xnor => vec![!(inp[0].clone() ^ inp[1].clone())],
            NodeFnType::Not  => vec![!inp[0].clone()],
        }
    }
}
