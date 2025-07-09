use std::collections::{HashMap, HashSet};
use std::ops::{Index, IndexMut};

use slotmap::{new_key_type, SlotMap};

use crate::bitarray::BitArray;
use crate::node::{Component, ComponentFn, PortProperties, PortType, PortUpdate};


new_key_type! {
    pub struct ValueKey;
    pub struct FunctionKey;
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug)]
pub struct Port {
    gate: FunctionKey,
    index: usize
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueIssue {
    ShortCircuit,
    MismatchedBitsizes { val_size: u8, port_size: u8, port: Port, is_input: bool },
    OscillationDetected
}
struct ValueNode {
    value: BitArray,
    links: HashSet<Port>,
    issues: HashSet<ValueIssue>
}
impl ValueNode {
    fn new(value: BitArray) -> Self {
        Self { value, links: HashSet::new(), issues: HashSet::new() }
    }
    fn issues(&self) -> &HashSet<ValueIssue> {
        &self.issues
    }
    fn set(&mut self, new_val: BitArray) -> bool {
        let success = self.value.len() == new_val.len();
        if success {
            self.value = new_val;
        }
        success
    }
}
struct FunctionNode {
    func: ComponentFn,
    port_types: Vec<PortType>,
    // connections
    links: Vec<Option<ValueKey>>,
    // value update
    old_values: Vec<BitArray>,
    values: Vec<BitArray>
}
impl FunctionNode {
    fn new(func: ComponentFn) -> Self {
        let ((links, port_types), (old_values, values)) = func.ports().into_iter()
            .map(|PortProperties { ty, bitsize }| ((None, ty), (BitArray::floating(bitsize), BitArray::floating(bitsize))))
            .unzip();
        
        let mut node = Self { func, links, port_types, old_values, values };
        node.func.initialize(&mut node.values);
        node
    }
}
#[derive(Default)]
struct Graph {
    values: SlotMap<ValueKey, ValueNode>,
    functions: SlotMap<FunctionKey, FunctionNode>,
}
impl Graph {
    pub fn add_value(&mut self, value: BitArray) -> ValueKey {
        self.values.insert(ValueNode::new(value))
    }
    pub fn add_function(&mut self, func: ComponentFn) -> FunctionKey {
        self.functions.insert(FunctionNode::new(func))
    }

    pub fn connect(&mut self, wire: ValueKey, port: Port) {
        self.disconnect(port);
        self.functions[port.gate].links[port.index].replace(wire);
        self.values[wire].links.insert(port);
    }
    pub fn disconnect(&mut self, port: Port) {
        let old_port = self.functions[port.gate].links[port.index];
        // If there was something there, remove it from the other side:
        if let Some(node) = old_port {
            let result = self.values[node].links.remove(&port);
            debug_assert!(result, "Gate should've been removed from assigned value node");
        }
    }
    pub fn clear_edges(&mut self, gate: FunctionKey) {
        self.functions[gate].links.iter_mut()
            .enumerate()
            .filter_map(|(index, p)| Some((index, p.take()?)))
            .for_each(|(index, node)| {
                let result = self.values[node].links.remove(&Port { gate, index });
                debug_assert!(result, "Gate should've been removed from assigned value node");
            });
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
pub struct Circuit {
    graph: Graph,
    transient: TransientState
}
#[derive(Clone, Copy)]
struct TriggerState {
    recalculate: bool
}
#[derive(Default)]
struct TransientState {
    triggers: HashMap<ValueKey, TriggerState>,
    frontier: HashSet<FunctionKey>
}

impl Circuit {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn add_value_node(&mut self, arr: BitArray) -> ValueKey {
        self.graph.add_value(arr)
    }
    pub fn add_function_node<F: Into<ComponentFn>>(&mut self, f: F) -> FunctionKey {
        self.graph.add_function(f.into())
    }
    pub fn connect(&mut self, wire: ValueKey, port: Port) {
        self.graph.connect(wire, port);
    }
    pub fn connect_all(&mut self, gate: FunctionKey, ports: &[ValueKey]) {
        self.graph.clear_edges(gate);
        ports.iter().copied()
            .enumerate()
            .for_each(|(index, wire)| self.connect(wire, Port { gate, index }));
    }
    pub fn run(&mut self, inputs: &[ValueKey]) {
        const RUN_ITER_LIMIT: usize = 10_000;

        self.transient.triggers = inputs.iter().copied()
            .map(|k| (k, TriggerState { recalculate: false }))
            .collect();
        self.transient.frontier.clear();

        let mut iteration = 0;
        while !self.transient.triggers.is_empty() || !self.transient.frontier.is_empty() {
            if iteration > RUN_ITER_LIMIT {
                for &key in self.transient.triggers.keys() {
                    self.graph[key].issues.insert(ValueIssue::OscillationDetected);
                }
                break;
            }
            // 1. Update circuit state at start of cycle, save functions to waken in frontier
            for (node, TriggerState { recalculate }) in std::mem::take(&mut self.transient.triggers) {
                self.graph[node].issues.clear(); // remove issues bc of update
                
                // Iterator of all port values
                // We join them using a join algorithm.
                // If the value changes after this join, 
                // then we know we should propagate the update to the function.

                let mut propagate_update = true;
                if recalculate {
                    // todo: assert right bitsize
                    // Get all port values feeding into value:
                    let it = self.graph[node].links.iter()
                        .filter(|&&Port { gate, index }| self.graph[gate].port_types[index].accepts_output())
                        .map(|&Port { gate, index }| self.graph[gate].values[index]);
                    // Join:
                    let result = it.clone()
                        .reduce(BitArray::join)
                        .unwrap_or_else(|| BitArray::floating(self[node].len())); // if no inputs, make bits floating
                    // Short circuit check:
                    if BitArray::short_circuits(it) {
                        self.graph[node].issues.insert(ValueIssue::ShortCircuit);
                    }

                    propagate_update = self[node] != result;
                    self.graph[node].value = result;
                }

                if propagate_update {
                    self.transient.frontier.extend({
                        self.graph[node].links.iter()
                            .filter(|&&Port { gate, index }| self.graph[gate].port_types[index].accepts_input())
                            .map(|&Port { gate, index: _ }| gate)
                    });
                }
            }
            // 2. For all functions to waken, apply function and save triggers for next cycle
            for gate_idx in std::mem::take(&mut self.transient.frontier) {
                let gate = &mut self.graph.functions[gate_idx];

                // Update inputs:
                gate.old_values = gate.values.clone();
                for (index, ((&port, port_value), port_type)) in std::iter::zip(&gate.links, &mut gate.values).zip(&gate.port_types).enumerate() {
                    if matches!(port_type, PortType::Output) { continue; }
                    // Only update inputs and inouts
                    let port_size = port_value.len();
                    let input = match port {
                        Some(n) => {
                            let node = &mut self.graph.values[n];
                            let val_size = node.value.len();
                            match val_size == port_size {
                                true  => node.value,
                                false => {
                                    node.issues.insert(ValueIssue::MismatchedBitsizes {
                                        val_size,
                                        port_size,
                                        port: Port { gate: gate_idx, index },
                                        is_input: true
                                    });
                                    BitArray::floating(port_size)
                                }
                            }
                        },
                        None => BitArray::floating(port_size)
                    };
                    
                    *port_value = input;
                }
                
                for PortUpdate { index, value } in gate.func.run(&gate.old_values, &gate.values) {
                    debug_assert!(self.graph[gate_idx].port_types[index].accepts_output(), "Input port cannot be updated");
                    if self.graph[gate_idx].values[index] != value {
                        self.graph[gate_idx].values[index] = value; // todo: assert right bitsize
                        
                        if let Some(sink_idx) = self.graph[gate_idx].links[index] {
                            self.transient.triggers.insert(sink_idx, TriggerState { recalculate: true });
                        }
                    }
                }
            }

            iteration += 1;
        }
    }

    pub fn get_issues(&self, key: ValueKey) -> &HashSet<ValueIssue> {
        self.graph[key].issues()
    }
    pub fn try_set(&mut self, key: ValueKey, val: BitArray) -> Result<(), ()> {
        match self.graph[key].set(val) {
            true => Ok(()),
            false => Err(())
        }
    }
    pub fn set(&mut self, key: ValueKey, val: BitArray) {
        self.try_set(key, val)
            .expect("Tried to set value with wrong bitsize")
    }
}
impl Index<ValueKey> for Circuit {
    type Output = BitArray;

    fn index(&self, index: ValueKey) -> &Self::Output {
        &self.graph[index].value
    }
}
impl Index<FunctionKey> for Circuit {
    type Output = ComponentFn;

    fn index(&self, index: FunctionKey) -> &Self::Output {
        &self.graph[index].func
    }
}