
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::ops::{Index, IndexMut};

use bitarray::{BitArray, BitState};
use node::{Component, Node, NodeFnType, PortTrigger};
use petgraph::csr::DefaultIx;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::{Directed, Direction, Graph};

pub mod bitarray;
pub mod node;

type CircuitIndex = DefaultIx;

#[derive(Default)]
struct Circuit {
    graph: Graph<Node, Edge, Directed, CircuitIndex>,
    inputs: Vec<ValueIx>,
    outputs: Vec<ValueIx>,
    transient: TransientState
}
#[derive(Default)]
struct TransientState {
    triggers: HashMap<ValueIx, BitArray>,
    frontier: HashSet<FunctionIx>
}

#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub struct ValueIx(NodeIndex<CircuitIndex>);
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub struct FunctionIx(NodeIndex<CircuitIndex>);

impl Circuit {
    fn new() -> Self {
        Default::default()
    }
    fn add_value_node(&mut self, arr: BitArray) -> ValueIx {
        ValueIx(self.graph.add_node(Node::Value(arr)))
    }
    fn add_input_node(&mut self, arr: BitArray) -> ValueIx {
        let ix = self.add_value_node(arr);
        self.inputs.push(ix);
        ix
    }
    fn add_output_node(&mut self, len: u8) -> ValueIx {
        let ix = self.add_value_node(BitArray::repeat(BitState::Imped, len));
        self.outputs.push(ix);
        ix
    }
    fn add_function_node(&mut self, f: NodeFnType) -> FunctionIx {
        FunctionIx(self.graph.add_node(Node::Function(f)))
    }
    fn inputs(&self) -> &[ValueIx] {
        &self.inputs
    }
    fn outputs(&self) -> &[ValueIx] {
        &self.outputs
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
        for (i, val) in std::iter::zip(self.inputs().to_vec(), values) {
            self[i] = val;
        }
    }
    fn get_outputs(&self) -> Vec<BitArray> {
        self.outputs().iter()
            .map(|&n| self[n].clone())
            .collect()
    }
    fn run(&mut self) {
        self.transient.triggers = self.inputs().iter()
            .map(|&n| (n, self[n].clone()))
            .collect();
        self.transient.frontier.clear();

        while !self.transient.triggers.is_empty() || !self.transient.frontier.is_empty() {
            // 1. Update circuit state at start of cycle, save functions to waken in frontier
            for (node, value) in std::mem::take(&mut self.transient.triggers) {
                self[node] = value;
                self.transient.frontier.extend({
                    self.graph.neighbors(node.0).map(FunctionIx)
                });
            }
            // 2. For all functions to waken, apply function and save triggers for next cycle
            for node in std::mem::take(&mut self.transient.frontier) {
                let mut nodes: Vec<_> = self.graph.edges_directed(node.0, Direction::Incoming)
                    .map(|e| (ValueIx(e.source()), *e.weight()))
                    .collect();
                nodes.sort_by_key(|&(_, i)| i);
                let inputs: Vec<_> = nodes.into_iter()
                    .map(|(i, _)| self[i].clone())
                    .collect();
                
                for PortTrigger { port, value } in self[node].run(&inputs) {
                    // todo: optimize
                    let val_node = self.graph.edges(node.0)
                        .find(|e| *e.weight() == port)
                        .map(|e| ValueIx(e.target()))
                        .unwrap();
                    
                    // Don't trigger if value didn't change
                    // todo: this could be put in the trigger logic
                    if self[val_node] != value {
                        match self.transient.triggers.entry(val_node) {
                            Entry::Occupied(e) if e.get() == &value => {},
                            Entry::Occupied(_) => todo!("short circuit"),
                            Entry::Vacant(e) => { e.insert(value); },
                        }
                    }
                }
            }
        }
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

        let wires = [
            circuit.add_input_node(BitArray::from_u64(a)),
            circuit.add_input_node(BitArray::from_u64(b)),
            circuit.add_output_node(64),
        ];
        let gates = [circuit.add_function_node(NodeFnType::Xor)];

        circuit.connect(gates[0], &[wires[0], wires[1]], &[wires[2]]);
        circuit.run();

        let left = a ^ b;
        let right = circuit.get_outputs()[0].to_u64().unwrap();
        assert_eq!(left, right, "0x{left:016X} != 0x{right:016X}");
    }

    #[test]
    fn dual() {
        let mut circuit = Circuit::new();
        let a = 0x9A3B2174_94093211;
        let b = 0x19182934_19AFFC94;
        let c = 0x92821734_182A9A9A;
        let d = 0xA8293129_FC03919D;

        let wires = [
            circuit.add_input_node(BitArray::from_u64(a)),
            circuit.add_input_node(BitArray::from_u64(b)),
            circuit.add_input_node(BitArray::from_u64(c)),
            circuit.add_input_node(BitArray::from_u64(d)),
            circuit.add_value_node(BitArray::repeat(BitState::Imped, 64)),
            circuit.add_value_node(BitArray::repeat(BitState::Imped, 64)),
            circuit.add_output_node(64),
        ];
        let gates = [
            circuit.add_function_node(NodeFnType::Xor),
            circuit.add_function_node(NodeFnType::Xor),
            circuit.add_function_node(NodeFnType::Xor),
        ];
        circuit.connect(gates[0], &[wires[0], wires[1]], &[wires[4]]);
        circuit.connect(gates[1], &[wires[2], wires[3]], &[wires[5]]);
        circuit.connect(gates[2], &[wires[4], wires[5]], &[wires[6]]);
        circuit.run();

        let left = a ^ b ^ c ^ d;
        let right = circuit.get_outputs()[0].to_u64().unwrap();
        assert_eq!(left, right, "0x{left:016X} != 0x{right:016X}");
    }

    #[test]
    fn metastable() {
        let mut circuit = Circuit::new();
        let a = 0x98A85409_19182A9F;

        let wires = [
            circuit.add_input_node(BitArray::from_u64(a)),
            circuit.add_output_node(64),
        ];
        let gates = [
            circuit.add_function_node(NodeFnType::Not),
            circuit.add_function_node(NodeFnType::Not),
        ];

        circuit.connect(gates[0], &[wires[0]], &[wires[1]]);
        circuit.connect(gates[1], &[wires[1]], &[wires[0]]);
        circuit.run();

        let (l1, r1) = (a, circuit[wires[0]].to_u64().unwrap());
        let (l2, r2) = (!a, circuit[wires[1]].to_u64().unwrap());
        assert_eq!(l1, r1, "0x{l1:016X} != 0x{r1:016X}");
        assert_eq!(l2, r2, "0x{l2:016X} != 0x{r2:016X}");
    }

    #[test]
    fn nand_propagate() {
        let mut circuit = Circuit::new();
        let wires = [
            circuit.add_input_node(BitArray::from_iter([BitState::Low])),
            circuit.add_value_node(BitArray::from_iter([BitState::Imped])),
            circuit.add_value_node(BitArray::from_iter([BitState::Imped])),
        ];
        let gates = [
            circuit.add_function_node(NodeFnType::Nand),
            circuit.add_function_node(NodeFnType::Nand),
        ];

        circuit.connect(gates[0], &[wires[0], wires[1]], &[wires[2]]);
        circuit.connect(gates[1], &[wires[0], wires[2]], &[wires[1]]);
        circuit.run();

        assert_eq!(0, circuit[wires[0]].to_u64().unwrap());
        assert_eq!(1, circuit[wires[1]].to_u64().unwrap());
        assert_eq!(1, circuit[wires[2]].to_u64().unwrap());
    }
}
