
use std::ops::{Index, IndexMut};

use bitarray::BitArray;
use node::{Node, NodeFnType};
use petgraph::csr::DefaultIx;
use petgraph::graph::NodeIndex;
use petgraph::{Directed, Direction, Graph};

pub mod bitarray;
pub mod node;

type CircuitIndex = DefaultIx;

#[derive(Default)]
struct Circuit {
    graph: Graph<Node, Edge, Directed, CircuitIndex>,
}
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct ValueIx(NodeIndex<CircuitIndex>);
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct FunctionIx(NodeIndex<CircuitIndex>);

impl Circuit {
    fn new() -> Self {
        Default::default()
    }
    fn add_value_node(&mut self, arr: BitArray) -> ValueIx {
        ValueIx(self.graph.add_node(Node::Value(arr)))
    }
    fn add_function_node(&mut self, f: NodeFnType) -> FunctionIx {
        FunctionIx(self.graph.add_node(Node::Function(f)))
    }
    fn inputs(&self) -> impl Iterator<Item=ValueIx> {
        self.graph.externals(Direction::Incoming).map(ValueIx)
    }
    fn outputs(&self) -> impl Iterator<Item=ValueIx> {
        self.graph.externals(Direction::Outgoing).map(ValueIx)
    }

    fn connect_in(&mut self, gate: FunctionIx, source: ValueIx, port: Edge) {
        self.graph.add_edge(source.0, gate.0, port);
    }
    fn connect_out(&mut self, gate: FunctionIx, sink: ValueIx, port: Edge) {
        self.graph.add_edge(gate.0, sink.0, port);
    }
    fn connect(&mut self, gate: FunctionIx, inputs: &[ValueIx], outputs: &[ValueIx]) {
        let incoming = self.graph.neighbors_directed(gate.0, Direction::Incoming).count();
        let outgoing = self.graph.neighbors_directed(gate.0, Direction::Outgoing).count();
        inputs.iter()
            .zip(incoming..)
            .for_each(|(&wire, port)| self.connect_in(gate, wire, port));
        outputs.iter()
            .zip(outgoing..)
            .for_each(|(&wire, port)| self.connect_out(gate, wire, port));
    }

    fn set_inputs(&mut self, values: Vec<BitArray>) {
        for (i, val) in std::iter::zip(Vec::from_iter(self.inputs()), values) {
            self[i] = val;
        }
    }
    fn get_outputs(&self) -> Vec<BitArray> {
        self.outputs()
            .map(|n| self[n].clone())
            .collect()
    }
    fn run(&mut self) {
        // TODO
    }
}
type Edge = usize; // port
impl Index<ValueIx> for Circuit {
    type Output = BitArray;

    fn index(&self, index: ValueIx) -> &Self::Output {
        match &self.graph[index.0] {
            Node::Value(n) => n,
            Node::Function(_) => panic!("expected node with value index to be a value node"),
        }
    }
}
impl IndexMut<ValueIx> for Circuit {
    fn index_mut(&mut self, index: ValueIx) -> &mut Self::Output {
        match &mut self.graph[index.0] {
            Node::Value(n) => n,
            Node::Function(_) => panic!("expected node with value index to be a value node"),
        }
    }
}
impl Index<FunctionIx> for Circuit {
    type Output = NodeFnType;

    fn index(&self, index: FunctionIx) -> &Self::Output {
        match &self.graph[index.0] {
            Node::Value(_) => panic!("expected node with function index to be a function node"),
            Node::Function(f) => f,
        }
    }
}
impl IndexMut<FunctionIx> for Circuit {
    fn index_mut(&mut self, index: FunctionIx) -> &mut Self::Output {
        match &mut self.graph[index.0] {
            Node::Value(_) => panic!("expected node with function index to be a function node"),
            Node::Function(f) => f,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let mut circuit = Circuit::new();
        let a = 0x9A3B2174_94093211;
        let b = 0x19182934_19AFFC94;
        let nodes = (
            circuit.add_value_node(BitArray::from_u64(a)),
            circuit.add_value_node(BitArray::from_u64(b)),
            circuit.add_function_node(NodeFnType::Xor),
            circuit.add_value_node(BitArray::floating(64)),
        );

        circuit.connect(nodes.2, &[nodes.0, nodes.1], &[nodes.3]);
        circuit.run();

        let left = a ^ b;
        let right = circuit.get_outputs()[0].to_u64();
        assert_eq!(left, right, "0x{left:16X} != 0x{right:16X}");
    }

    #[test]
    fn dual() {
        let mut circuit = Circuit::new();
        let a = 0x9A3B2174_94093211;
        let b = 0x19182934_19AFFC94;
        let c = 0x92821734_182A9A9A;
        let d = 0xA8293129_FC03919D;
        let nodes = (
            circuit.add_value_node(BitArray::from_u64(a)),
            circuit.add_value_node(BitArray::from_u64(b)),
            circuit.add_value_node(BitArray::from_u64(c)),
            circuit.add_value_node(BitArray::from_u64(d)),
            circuit.add_function_node(NodeFnType::Xor),
            circuit.add_function_node(NodeFnType::Xor),
            circuit.add_function_node(NodeFnType::Xor),
            circuit.add_value_node(BitArray::floating(64)),
            circuit.add_value_node(BitArray::floating(64)),
            circuit.add_value_node(BitArray::floating(64)),
        );
        circuit.connect(nodes.4, &[nodes.0, nodes.1], &[nodes.7]);
        circuit.connect(nodes.5, &[nodes.2, nodes.3], &[nodes.8]);
        circuit.connect(nodes.6, &[nodes.7, nodes.8], &[nodes.9]);
        circuit.run();

        let left = a ^ b ^ c ^ d;
        let right = circuit.get_outputs()[0].to_u64();
        assert_eq!(left, right, "0x{left:16X} != 0x{right:16X}");
    }
}
