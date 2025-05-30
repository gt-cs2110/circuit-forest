use crate::bitarray::BitArray;

pub trait Component {
    fn num_inputs(&self) -> usize;
    fn num_outputs(&self) -> usize;
    fn run(&mut self, inp: &[BitArray]) -> Vec<BitArray>;
}

pub struct Node {
    pub(crate) ty: NodeType,
    in_cache: Option<Vec<BitArray>>,
    out_cache: Option<Vec<BitArray>>,
}
impl Node {
    pub fn get_inputs(&self) -> Option<&[BitArray]> {
        self.in_cache.as_deref()
    }
    pub fn get_outputs(&self) -> Option<&[BitArray]> {
        self.out_cache.as_deref()
    }
    pub fn invalidate(&mut self) {
        self.in_cache.take();
        self.out_cache.take();
    }
}
impl From<NodeType> for Node {
    fn from(ty: NodeType) -> Self {
        Self { ty, in_cache: None, out_cache: None }
    }
}
impl Component for Node {
    fn num_inputs(&self) -> usize {
        self.ty.num_inputs()
    }

    fn num_outputs(&self) -> usize {
        self.ty.num_outputs()
    }

    fn run(&mut self, inp: &[BitArray]) -> Vec<BitArray> {
        // Load from cache
        if let Some(out) = &self.out_cache {
            return out.clone();
        }
        
        // Load to cache
        let result = self.ty.run(inp);
        self.in_cache.replace(inp.to_vec());
        self.out_cache.replace(result.clone());
        result
    }
}
pub enum NodeType {
    Input(BitArray), Output, And, Or, Xor, Nand, Nor, Xnor, Not
}
impl Component for NodeType {
    fn num_inputs(&self) -> usize {
        match self {
            NodeType::Input(_) => 0,
            NodeType::Output => 1,
            NodeType::And  => 2,
            NodeType::Or   => 2,
            NodeType::Xor  => 2,
            NodeType::Nand => 2,
            NodeType::Nor  => 2,
            NodeType::Xnor => 2,
            NodeType::Not  => 1,
        }
    }
    fn num_outputs(&self) -> usize {
        match self {
            NodeType::Input(_) => 1,
            NodeType::Output => 0,
            NodeType::And  => 1,
            NodeType::Or   => 1,
            NodeType::Xor  => 1,
            NodeType::Nand => 1,
            NodeType::Nor  => 1,
            NodeType::Xnor => 1,
            NodeType::Not  => 1,
        }
    }

    fn run(&mut self, inp: &[BitArray]) -> Vec<BitArray> {
        match self {
            NodeType::Input(s) => vec![s.clone()],
            NodeType::Output => vec![],
            NodeType::And  => vec![inp[0].clone() & inp[1].clone()],
            NodeType::Or   => vec![inp[0].clone() | inp[1].clone()],
            NodeType::Xor  => vec![inp[0].clone() ^ inp[1].clone()],
            NodeType::Nand => vec![!(inp[0].clone() & inp[1].clone())],
            NodeType::Nor  => vec![!(inp[0].clone() | inp[1].clone())],
            NodeType::Xnor => vec![!(inp[0].clone() ^ inp[1].clone())],
            NodeType::Not  => vec![!inp[0].clone()],
        }
    }
}
