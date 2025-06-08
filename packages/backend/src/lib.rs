
use std::collections::HashSet;
use std::ops::{Index, IndexMut};

use bitarray::BitArray;
use node::{Component, ComponentFn};
use slotmap::{new_key_type, SlotMap};

pub mod bitarray;
pub mod node;

new_key_type! {
    pub struct ValueKey;
    pub struct FunctionKey;
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
struct Port {
    gate: FunctionKey,
    index: usize
}
impl Port {
    fn new(gate: FunctionKey, index: usize) -> Self {
        Self { gate, index }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ValueIssue {
    ShortCircuit,
    MismatchedBitsizes { val_size: u8, port_size: u8, port: Port, is_input: bool },
    OscillationDetected
}
struct ValueNode {
    value: BitArray,
    inputs: HashSet<Port>,
    outputs: HashSet<Port>,
    issues: HashSet<ValueIssue>
}
struct FunctionNode {
    func: ComponentFn,
    inputs: Vec<Option<ValueKey>>,
    outputs: Vec<(Option<ValueKey>, BitArray)>,
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
            value, inputs: HashSet::new(), outputs: HashSet::new(), issues: HashSet::new()
        })
    }
    pub fn add_function(&mut self, func: ComponentFn) -> FunctionKey {
        let inputs = vec![None; func.input_sizes().len()];
        let outputs = func.output_sizes().into_iter()
            .map(|size| (None, BitArray::floating(size)))
            .collect();
        self.functions.insert(FunctionNode {
            func, inputs, outputs
        })
    }

    pub fn connect_in(&mut self, gate: FunctionKey, source: ValueKey, port: usize) {
        self.disconnect_in(gate, port);
        self.functions[gate].inputs[port].replace(source);
        self.values[source].outputs.insert(Port::new(gate, port));
    }
    pub fn connect_out(&mut self, gate: FunctionKey, sink: ValueKey, port: usize) {
        self.disconnect_out(gate, port);
        self.functions[gate].outputs[port].0.replace(sink);
        self.values[sink].inputs.insert(Port::new(gate, port));
    }

    pub fn disconnect_in(&mut self, gate: FunctionKey, port: usize) {
        let old_input = self.functions[gate].inputs[port].take();
        // If there was something there, remove it from the other side:
        if let Some(source) = old_input {
            let result = self.values[source].outputs.remove(&Port::new(gate, port));
            debug_assert!(result, "Gate should've been removed from source value's outputs");
        }
    }
    pub fn disconnect_out(&mut self, gate: FunctionKey, port: usize) {
        let old_output = self.functions[gate].outputs[port].0.take();
        // If there was something there, remove it from the other side:
        if let Some(sink) = old_output {
            let result = self.values[sink].inputs.remove(&Port::new(gate, port));
            debug_assert!(result, "Gate should've been removed from sink value's inputs");
        }
    }

    pub fn clear_inputs(&mut self, gate: FunctionKey) {
        self.functions[gate].inputs.iter_mut()
            .enumerate()
            .filter_map(|(port, inp)| Some((port, inp.take()?)))
            .for_each(|(port, source)| {
                let result = self.values[source].outputs.remove(&Port::new(gate, port));
                debug_assert!(result, "Gate should've been removed from source value's outputs");
            });
        }
    pub fn clear_outputs(&mut self, gate: FunctionKey) {
        self.functions[gate].outputs.iter_mut()
            .enumerate()
            .filter_map(|(port, outp)| Some((port, outp.0.take()?)))
            .for_each(|(port, sink)| {
                let result = self.values[sink].inputs.remove(&Port::new(gate, port));
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
    transient: TransientState
}
#[derive(Default)]
struct TransientState {
    triggers: HashSet<ValueKey>,
    frontier: HashSet<FunctionKey>
}

impl Circuit {
    fn new() -> Self {
        Default::default()
    }
    fn add_value_node(&mut self, arr: BitArray) -> ValueKey {
        self.graph.add_value(arr)
    }
    fn add_function_node<F: Into<ComponentFn>>(&mut self, f: F) -> FunctionKey {
        self.graph.add_function(f.into())
    }
    fn connect_one(&mut self, port: Port, as_input: bool, value: ValueKey) {
        match as_input {
            true  => self.graph.connect_in(port.gate, value, port.index),
            false => self.graph.connect_out(port.gate, value, port.index),
        }
    }
    fn connect(&mut self, gate: FunctionKey, inputs: &[ValueKey], outputs: &[ValueKey]) {
        self.graph.clear_edges(gate);
        inputs.iter().copied().enumerate().for_each(|(i, source)| self.graph.connect_in(gate, source, i));
        outputs.iter().copied().enumerate().for_each(|(i, sink)| self.graph.connect_out(gate, sink, i));
    }
    fn run(&mut self, inputs: &[ValueKey]) {
        const RUN_LIMIT: usize = 10_000;

        self.transient.triggers.clear();
        for &node in inputs {
            self.graph[node].issues.clear(); // remove issues bc of update
        }
        self.transient.frontier.extend(
            inputs.iter()
                .flat_map(|&n| &self.graph[n].outputs)
                .map(|&Port { gate, index: _ }| gate)
        );

        let mut iteration = 0;
        while !self.transient.triggers.is_empty() || !self.transient.frontier.is_empty() {
            if iteration > RUN_LIMIT {
                for key in self.transient.triggers.clone() {
                    self.graph[key].issues.insert(ValueIssue::ShortCircuit);
                }
                break;
            }
            // 1. Update circuit state at start of cycle, save functions to waken in frontier
            for node in std::mem::take(&mut self.transient.triggers) {
                self.graph[node].issues.clear(); // remove issues bc of update
                
                // Iterator of all port values
                // We join them using a join algorithm.
                // If the value changes after this join, 
                // then we know we should propagate the update to the function.

                // Get all port values:
                let it = self.graph[node].inputs.iter()
                    .map(|&Port { gate, index }| self.graph[gate].outputs[index].1); // all port values
                // Join:
                let result = it.clone()
                    .reduce(BitArray::join)
                    .unwrap_or_else(|| BitArray::floating(self[node].len())); // if no inputs, make bits floating
                // Short circuit check:
                if BitArray::short_circuits(it) {
                    self.graph[node].issues.insert(ValueIssue::ShortCircuit);
                }

                if self[node] != result {
                    self[node] = result;
                    self.transient.frontier.extend({
                        self.graph[node].outputs
                            .iter()
                            .map(|&Port { gate, index: _ }| gate)
                    });
                }
            }
            // 2. For all functions to waken, apply function and save triggers for next cycle
            for gate_idx in std::mem::take(&mut self.transient.frontier) {
                let gate = &self.graph.functions[gate_idx];
                let inputs: Vec<_> = std::iter::zip(gate.func.input_sizes(), gate.inputs.iter().enumerate())
                    .map(|(port_size, (port, &m_node))| match m_node {
                        Some(n) if self.graph.values[n].value.len() == port_size => self.graph.values[n].value,
                        Some(n) => {
                            let val_size = self.graph.values[n].value.len();
                            self.graph.values[n].issues.insert(ValueIssue::MismatchedBitsizes {
                                val_size,
                                port_size,
                                port: Port { gate: gate_idx, index: port },
                                is_input: true
                            });
                            BitArray::floating(port_size)
                        },
                        None => BitArray::floating(port_size),
                    })
                    .collect();
                
                for (port, value) in self[gate_idx].run(&inputs).into_iter().enumerate() {
                    let (Some(sink_idx), ref mut out_val) = self.graph[gate_idx].outputs[port] else { continue };
                    *out_val = value;
                    self.transient.triggers.insert(sink_idx);
                }
            }

            iteration += 1;
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
    type Output = ComponentFn;

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
    use crate::bitarray::BitState;

    use super::*;

    #[test]
    fn simple() {
        let mut circuit = Circuit::new();
        let a = 0x9A3B2174_94093211;
        let b = 0x19182934_19AFFC94;

        // Wires
        let a_in = circuit.add_value_node(BitArray::from(a));
        let b_in = circuit.add_value_node(BitArray::from(b));
        let out  = circuit.add_value_node(BitArray::floating(64));
        // Gates
        let gates = [circuit.add_function_node(node::Xor::new(64, 2))];

        circuit.connect(gates[0], &[a_in, b_in], &[out]);
        circuit.run(&[a_in, b_in]);

        let left = a ^ b;
        let right = u64::try_from(circuit[out]).unwrap();
        assert_eq!(left, right, "0x{left:016X} != 0x{right:016X}");
    }

    #[test]
    fn dual() {
        let mut circuit = Circuit::new();
        let a = 0x9A3B2174_94093211;
        let b = 0x19182934_19AFFC94;
        let c = 0x92821734_182A9A9A;
        let d = 0xA8293129_FC03919D;

        // Wires
        let a_in = circuit.add_value_node(BitArray::from(a));
        let b_in = circuit.add_value_node(BitArray::from(b));
        let c_in = circuit.add_value_node(BitArray::from(c));
        let d_in = circuit.add_value_node(BitArray::from(d));
        let ab_mid = circuit.add_value_node(BitArray::floating(64));
        let cd_mid = circuit.add_value_node(BitArray::floating(64));
        let out = circuit.add_value_node(BitArray::floating(64));
        // Gates
        let gates = [
            circuit.add_function_node(node::Xor::new(64, 2)),
            circuit.add_function_node(node::Xor::new(64, 2)),
            circuit.add_function_node(node::Xor::new(64, 2)),
        ];

        circuit.connect(gates[0], &[a_in, b_in], &[ab_mid]);
        circuit.connect(gates[1], &[c_in, d_in], &[cd_mid]);
        circuit.connect(gates[2], &[ab_mid, cd_mid], &[out]);
        circuit.run(&[a_in, b_in, c_in, d_in]);

        let left = a ^ b ^ c ^ d;
        let right = u64::try_from(circuit[out]).unwrap();
        assert_eq!(left, right, "0x{left:016X} != 0x{right:016X}");
    }

    #[test]
    fn metastable() {
        let mut circuit = Circuit::new();
        let a = 0x98A85409_19182A9F;

        let wires = [
            circuit.add_value_node(BitArray::from(a)),
            circuit.add_value_node(BitArray::floating(64)),
        ];
        let gates = [
            circuit.add_function_node(node::Not::new(64)),
            circuit.add_function_node(node::Not::new(64)),
        ];

        circuit.connect(gates[0], &[wires[0]], &[wires[1]]);
        circuit.connect(gates[1], &[wires[1]], &[wires[0]]);
        circuit.run(&[wires[0]]);

        let (l1, r1) = (a, u64::try_from(circuit[wires[0]]).unwrap());
        let (l2, r2) = (!a, u64::try_from(circuit[wires[1]]).unwrap());
        assert_eq!(l1, r1, "0x{l1:016X} != 0x{r1:016X}");
        assert_eq!(l2, r2, "0x{l2:016X} != 0x{r2:016X}");
    }

    #[test]
    fn nand_propagate() {
        let mut circuit = Circuit::new();

        let inp  = circuit.add_value_node(BitArray::from_iter([BitState::Low]));
        let out0 = circuit.add_value_node(BitArray::from_iter([BitState::Imped]));
        let out1 = circuit.add_value_node(BitArray::from_iter([BitState::Imped]));
        let gates = [
            circuit.add_function_node(node::Nand::new(1, 2)),
            circuit.add_function_node(node::Nand::new(1, 2)),
        ];

        circuit.connect(gates[0], &[inp, out1], &[out0]);
        circuit.connect(gates[1], &[inp, out0], &[out1]);
        circuit.run(&[inp]);

        assert_eq!(0, u64::try_from(circuit[inp]).unwrap());
        assert_eq!(1, u64::try_from(circuit[out0]).unwrap());
        assert_eq!(1, u64::try_from(circuit[out1]).unwrap());
    }

    #[test]
    fn conflict_pass_z() {
        let mut circuit = Circuit::new();

        // Wires
        let lo = circuit.add_value_node(BitArray::from_iter([BitState::Low]));
        let hi = circuit.add_value_node(BitArray::from_iter([BitState::High]));
        let out = circuit.add_value_node(BitArray::from_iter([BitState::Unk]));
        // Gates
        let gates = [
            circuit.add_function_node(node::TriState::new(1)),
            circuit.add_function_node(node::TriState::new(1)),
        ];

        circuit.connect(gates[0], &[lo, lo], &[out]);
        circuit.connect(gates[1], &[hi, hi], &[out]);
        circuit.run(&[lo, hi]);

        assert_eq!(1, u64::try_from(circuit[out]).unwrap());
    }

    #[test]
    fn conflict_fail() {
        let mut circuit = Circuit::new();
        
        // Wires
        let lo = circuit.add_value_node(BitArray::from_iter([BitState::Low]));
        let hi = circuit.add_value_node(BitArray::from_iter([BitState::High]));
        let out = circuit.add_value_node(BitArray::from_iter([BitState::Unk]));
        // Gates
        let gates = [
            circuit.add_function_node(node::TriState::new(1)),
            circuit.add_function_node(node::TriState::new(1)),
        ];

        circuit.connect(gates[0], &[hi, lo], &[out]);
        circuit.connect(gates[1], &[hi, hi], &[out]);
        circuit.run(&[lo, hi]);

        assert!(circuit.graph[out].issues.contains(&ValueIssue::ShortCircuit), "Node 'out' should short circuit");
    }

    #[test]
    fn delay_conflict() {
        let mut circuit = Circuit::new();
        
        // Wires
        let inp = circuit.add_value_node(BitArray::from_iter([BitState::Low])); 
        let mid = circuit.add_value_node(BitArray::from_iter([BitState::Low]));
        let out = circuit.add_value_node(BitArray::from_iter([BitState::Low]));
        // Gates
        let gates = [
            circuit.add_function_node(node::Not::new(1)),
            circuit.add_function_node(node::Not::new(1)),
            circuit.add_function_node(node::Not::new(1)),
        ];

        circuit.connect(gates[0], &[inp], &[mid]);
        circuit.connect(gates[1], &[mid], &[out]);
        circuit.connect(gates[2], &[inp], &[out]);
        circuit.run(&[inp]);

        assert!(circuit.graph[out].issues.contains(&ValueIssue::ShortCircuit), "Node 'out' should short circuit");
    }

    #[test]
    fn rs_latch() {
        let mut circuit = Circuit::new();
        let [r, s, q, qp] = [
            circuit.add_value_node(BitArray::from_iter([BitState::High])), // R
            circuit.add_value_node(BitArray::from_iter([BitState::High])), // S
            circuit.add_value_node(BitArray::from_iter([BitState::High])), // Q
            circuit.add_value_node(BitArray::from_iter([BitState::Low])), // Q'
        ];
        let [rnand, snand] = [
            circuit.add_function_node(node::Nand::new(1, 2)),
            circuit.add_function_node(node::Nand::new(1, 2)),
        ];

        // R = 1, S = 1
        circuit.connect(rnand, &[r, q], &[qp]);
        circuit.connect(snand, &[s, qp], &[q]);
        circuit.run(&[r, s]);

        assert_eq!(u64::try_from(circuit[q]).unwrap(), 1);
        assert_eq!(u64::try_from(circuit[qp]).unwrap(), 0);
        
        // R = 0, S = 1
        circuit[r] = BitArray::from_iter([BitState::Low]); // TODO, don't allow this
        circuit.run(&[r]);

        assert_eq!(u64::try_from(circuit[q]).unwrap(), 0);
        assert_eq!(u64::try_from(circuit[qp]).unwrap(), 1);

        // R = 1, S = 0
        circuit[r] = BitArray::from_iter([BitState::High]);
        circuit[s] = BitArray::from_iter([BitState::Low]);
        circuit.run(&[r, s]);

        assert_eq!(u64::try_from(circuit[q]).unwrap(), 1);
        assert_eq!(u64::try_from(circuit[qp]).unwrap(), 0);
    }

    #[test]
    fn chain() {
        let mut circuit = Circuit::new();
        
        // Wires
        let a_in = circuit.add_value_node(BitArray::from_iter([BitState::High]));
        let b_in = circuit.add_value_node(BitArray::from_iter([BitState::Low]));
        let c_in = circuit.add_value_node(BitArray::from_iter([BitState::High]));
        let d_in = circuit.add_value_node(BitArray::from_iter([BitState::Low]));
        let e_in = circuit.add_value_node(BitArray::from_iter([BitState::High]));
        let ab_mid = circuit.add_value_node(BitArray::floating(1));
        let abc_mid = circuit.add_value_node(BitArray::floating(1));
        let abcd_mid = circuit.add_value_node(BitArray::floating(1));
        let out = circuit.add_value_node(BitArray::floating(1));
        // Gates
        let gates = [
            circuit.add_function_node(node::Xor::new(1, 2)),
            circuit.add_function_node(node::Xor::new(1, 2)),
            circuit.add_function_node(node::Xor::new(1, 2)),
            circuit.add_function_node(node::Xor::new(1, 2)),
        ];

        circuit.connect(gates[0], &[a_in, b_in], &[ab_mid]);
        circuit.connect(gates[1], &[ab_mid, c_in], &[abc_mid]);
        circuit.connect(gates[2], &[abc_mid, d_in], &[abcd_mid]);
        circuit.connect(gates[3], &[abcd_mid, e_in], &[out]);
        circuit.run(&[a_in, b_in, c_in, d_in, e_in]);

        assert_eq!(u64::try_from(circuit[out]).unwrap(), 1);
    }
}
