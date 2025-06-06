
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::ops::{Index, IndexMut};

use bitarray::{BitArray, BitState};
use node::{Component, NodeFnType, PortTrigger};
use slotmap::{new_key_type, SlotMap};

pub mod bitarray;
pub mod node;

new_key_type! {
    pub struct ValueKey;
    pub struct FunctionKey;
}
struct ValueNode {
    value: BitArray,
    inputs: HashSet<FunctionKey>,
    outputs: HashSet<FunctionKey>,
}
struct FunctionNode {
    func: NodeFnType,
    inputs: Vec<Option<ValueKey>>,
    outputs: Vec<Option<ValueKey>>,
}
#[derive(Default)]
struct Graph {
    values: SlotMap<ValueKey, ValueNode>,
    functions: SlotMap<FunctionKey, FunctionNode>,
}
impl Graph {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_value(&mut self, value: BitArray) -> ValueKey {
        self.values.insert(ValueNode {
            value, inputs: HashSet::new(), outputs: HashSet::new()
        })
    }
    pub fn add_function(&mut self, func: NodeFnType) -> FunctionKey {
        let inputs = func.inputs().len();
        let outputs = func.outputs().len();
        self.functions.insert(FunctionNode {
            func, inputs: vec![None; inputs], outputs: vec![None; outputs]
        })
    }

    pub fn connect_in(&mut self, gate: FunctionKey, source: ValueKey, port: usize) {
        self.disconnect_in(gate, port);
        self.functions[gate].inputs[port].replace(source);
        self.values[source].outputs.insert(gate);
    }
    pub fn connect_out(&mut self, gate: FunctionKey, sink: ValueKey, port: usize) {
        self.disconnect_out(gate, port);
        self.functions[gate].outputs[port].replace(sink);
        self.values[sink].inputs.insert(gate);
    }

    pub fn disconnect_in(&mut self, gate: FunctionKey, port: usize) {
        let old_input = self.functions[gate].inputs[port].take();
        // If there was something there, remove it from the other side:
        if let Some(source) = old_input {
            let result = self.values[source].outputs.remove(&gate);
            debug_assert!(result, "Gate should've been removed from source value's outputs");
        }
    }
    pub fn disconnect_out(&mut self, gate: FunctionKey, port: usize) {
        let old_output = self.functions[gate].outputs[port].take();
        // If there was something there, remove it from the other side:
        if let Some(sink) = old_output {
            let result = self.values[sink].inputs.remove(&gate);
            debug_assert!(result, "Gate should've been removed from sink value's inputs");
        }
    }

    pub fn clear_inputs(&mut self, gate: FunctionKey) {
        self.functions[gate].inputs.iter_mut()
            .filter_map(|inp| inp.take())
            .for_each(|source| {
                let result = self.values[source].outputs.remove(&gate);
                debug_assert!(result, "Gate should've been removed from source value's outputs");
            });
        }
    pub fn clear_outputs(&mut self, gate: FunctionKey) {
        self.functions[gate].outputs.iter_mut()
            .filter_map(|outp| outp.take())
            .for_each(|sink| {
                let result = self.values[sink].inputs.remove(&gate);
                debug_assert!(result, "Gate should've been removed from sink value's inputs");
            });

    }
    pub fn clear_edges(&mut self, gate: FunctionKey) {
        self.clear_inputs(gate);
        self.clear_outputs(gate);
    }
}
impl Index<ValueKey> for Graph {
    type Output = ValueNode;

    fn index(&self, index: ValueKey) -> &Self::Output {
        &self.values[index]
    }
}
impl IndexMut<ValueKey> for Graph {
    fn index_mut(&mut self, index: ValueKey) -> &mut Self::Output {
        &mut self.values[index]
    }
}
impl Index<FunctionKey> for Graph {
    type Output = FunctionNode;

    fn index(&self, index: FunctionKey) -> &Self::Output {
        &self.functions[index]
    }
}
impl IndexMut<FunctionKey> for Graph {
    fn index_mut(&mut self, index: FunctionKey) -> &mut Self::Output {
        &mut self.functions[index]
    }
}
#[derive(Default)]
struct Circuit {
    graph: Graph,
    inputs: Vec<ValueKey>,
    outputs: Vec<ValueKey>,
    transient: TransientState
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum PortType { Input, Output }
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Port {
    ty: PortType,
    gate: FunctionKey,
    index: usize
}
#[derive(Default)]
struct TransientState {
    triggers: HashMap<ValueKey, BitArray>,
    frontier: HashSet<FunctionKey>
}

impl Circuit {
    fn new() -> Self {
        Default::default()
    }
    fn add_value_node(&mut self, arr: BitArray) -> ValueKey {
        self.graph.add_value(arr)
    }
    fn add_input_node(&mut self, arr: BitArray) -> ValueKey {
        let ix = self.graph.add_value(arr);
        self.inputs.push(ix);
        ix
    }
    fn add_output_node(&mut self, len: u8) -> ValueKey {
        let ix = self.graph.add_value(BitArray::repeat(BitState::Imped, len));
        self.outputs.push(ix);
        ix
    }
    fn add_function_node(&mut self, f: NodeFnType) -> FunctionKey {
        self.graph.add_function(f)
    }
    fn inputs(&self) -> &[ValueKey] {
        &self.inputs
    }
    fn outputs(&self) -> &[ValueKey] {
        &self.outputs
    }

    fn connect_one(&mut self, port: Port, value: ValueKey) {
        match port {
            Port { ty: PortType::Input,  gate, index } => self.graph.connect_in(gate, value, index),
            Port { ty: PortType::Output, gate, index } => self.graph.connect_out(gate, value, index),
        }
    }
    fn connect(&mut self, gate: FunctionKey, inputs: &[ValueKey], outputs: &[ValueKey]) {
        self.graph.clear_edges(gate);
        inputs.iter().copied().enumerate().for_each(|(i, source)| self.graph.connect_in(gate, source, i));
        outputs.iter().copied().enumerate().for_each(|(i, sink)| self.graph.connect_out(gate, sink, i));
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
                self.transient.frontier.extend(self.graph[node].outputs.iter().copied());
            }
            // 2. For all functions to waken, apply function and save triggers for next cycle
            for gate_idx in std::mem::take(&mut self.transient.frontier) {
                let gate = &self.graph[gate_idx];
                let inputs: Vec<_> = std::iter::zip(gate.func.inputs(), gate.inputs.iter())
                    .map(|(size, &m_node)| match m_node {
                        Some(n) => self.graph[n].value.clone(),
                        None => BitArray::repeat(BitState::Imped, size),
                    })
                    .collect();
                
                for PortTrigger { port, value } in self[gate_idx].run(&inputs) {
                    let Some(sink_idx) = self.graph[gate_idx].outputs[port] else { continue };
                    // Only trigger if value changed
                    if self[sink_idx] != value {
                        match self.transient.triggers.entry(sink_idx) {
                            Entry::Occupied(mut e) => {
                                let join = BitArray::try_join(e.get().clone(), value.clone());
                                match join {
                                    Some(w) => { e.insert(w); },
                                    None => todo!("short circuit {:?}!={:?}", e.get(), value)
                                }
                            },
                            Entry::Vacant(e) => { e.insert(value); },
                        }
                    }
                }
            }
        }
    }
}
impl Index<ValueKey> for Circuit {
    type Output = BitArray;

    fn index(&self, index: ValueKey) -> &Self::Output {
        &self.graph[index].value
    }
}
impl IndexMut<ValueKey> for Circuit {
    fn index_mut(&mut self, index: ValueKey) -> &mut Self::Output {
        &mut self.graph[index].value
    }
}
impl Index<FunctionKey> for Circuit {
    type Output = NodeFnType;

    fn index(&self, index: FunctionKey) -> &Self::Output {
        &self.graph[index].func
    }
}
impl IndexMut<FunctionKey> for Circuit {
    fn index_mut(&mut self, index: FunctionKey) -> &mut Self::Output {
        &mut self.graph[index].func
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
            circuit.add_input_node(BitArray::from(a)),
            circuit.add_input_node(BitArray::from(b)),
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
            circuit.add_input_node(BitArray::from(a)),
            circuit.add_input_node(BitArray::from(b)),
            circuit.add_input_node(BitArray::from(c)),
            circuit.add_input_node(BitArray::from(d)),
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
            circuit.add_input_node(BitArray::from(a)),
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

    #[test]
    fn conflict_pass_z() {
        let mut circuit = Circuit::new();
        let wires = [
            circuit.add_input_node(BitArray::from_iter([BitState::Low])),
            circuit.add_input_node(BitArray::from_iter([BitState::High])),
            circuit.add_value_node(BitArray::from_iter([BitState::Unk])),
        ];
        let gates = [
            circuit.add_function_node(NodeFnType::TriState),
            circuit.add_function_node(NodeFnType::TriState),
        ];
        circuit.connect(gates[0], &[wires[0], wires[0]], &[wires[2]]);
        circuit.connect(gates[1], &[wires[1], wires[1]], &[wires[2]]);
        circuit.run();

        assert_eq!(1, circuit[wires[2]].to_u64().unwrap());
    }

    #[test]
    #[should_panic]
    fn conflict_fail() {
        let mut circuit = Circuit::new();
        let wires = [
            circuit.add_input_node(BitArray::from_iter([BitState::Low])),
            circuit.add_input_node(BitArray::from_iter([BitState::High])),
            circuit.add_value_node(BitArray::from_iter([BitState::Unk])),
        ];
        let gates = [
            circuit.add_function_node(NodeFnType::TriState),
            circuit.add_function_node(NodeFnType::TriState),
        ];
        circuit.connect(gates[0], &[wires[1], wires[0]], &[wires[2]]);
        circuit.connect(gates[1], &[wires[1], wires[1]], &[wires[2]]);
        circuit.run();
    }

    #[test]
    fn rs_latch() {
        let mut circuit = Circuit::new();
        let [r, s, q, qp] = [
            circuit.add_input_node(BitArray::from_iter([BitState::High])), // R
            circuit.add_input_node(BitArray::from_iter([BitState::High])), // S
            circuit.add_value_node(BitArray::from_iter([BitState::High])), // Q
            circuit.add_value_node(BitArray::from_iter([BitState::Low])), // Q'
        ];
        let [rnand, snand] = [
            circuit.add_function_node(NodeFnType::Nand),
            circuit.add_function_node(NodeFnType::Nand),
        ];

        // R = 1, S = 1
        circuit.connect(rnand, &[r, q], &[qp]);
        circuit.connect(snand, &[s, qp], &[q]);
        circuit.run();

        assert_eq!(circuit[q].to_u64().unwrap(), 1);
        assert_eq!(circuit[qp].to_u64().unwrap(), 0);
        
        // R = 0, S = 1
        circuit.set_inputs(vec![BitArray::from_iter([BitState::Low]), BitArray::from_iter([BitState::High])]);
        circuit.run();

        assert_eq!(circuit[q].to_u64().unwrap(), 0);
        assert_eq!(circuit[qp].to_u64().unwrap(), 1);

        // R = 1, S = 0
        circuit.set_inputs(vec![BitArray::from_iter([BitState::High]), BitArray::from_iter([BitState::Low])]);
        circuit.run();

        assert_eq!(circuit[q].to_u64().unwrap(), 1);
        assert_eq!(circuit[qp].to_u64().unwrap(), 0);
    }
}
