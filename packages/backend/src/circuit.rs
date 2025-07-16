mod state;

use std::collections::HashSet;
use std::ops::{Index, IndexMut};

use slotmap::{new_key_type, SlotMap};

use crate::bitarray::BitArray;
use crate::circuit::state::{index_mut, CircuitState, TriggerState, ValueState};
use crate::node::{Component, ComponentFn, PortType, PortUpdate};


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

#[derive(Default)]
struct ValueNode {
    links: HashSet<Port>
}
impl ValueNode {
    fn new() -> Self {
        Default::default()
    }
}
struct FunctionNode {
    func: ComponentFn,
    port_types: Vec<PortType>,
    // connections
    links: Vec<Option<ValueKey>>,
}
impl FunctionNode {
    fn new(func: ComponentFn) -> Self {
        let port_types: Vec<_> = func.ports()
            .into_iter()
            .map(|props| props.ty)
            .collect();
        let links = vec![None; port_types.len()];
        
        Self { func, links, port_types }
    }
}
#[derive(Default)]
struct Graph {
    values: SlotMap<ValueKey, ValueNode>,
    functions: SlotMap<FunctionKey, FunctionNode>,
}
impl Graph {
    pub fn add_value(&mut self) -> ValueKey {
        self.values.insert(ValueNode::new())
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
    state: CircuitState
}

impl Circuit {
    pub fn new() -> Self {
        Default::default()
    }
    #[deprecated]
    pub fn add_value_node(&mut self, arr: BitArray) -> ValueKey {
        let key = self.add_value_node2();
        self.state.values.insert(key, ValueState::new(arr));
        key
    }
    pub fn add_value_node2(&mut self) -> ValueKey {
        let key = self.graph.add_value();
        self.state.init_value(key);
        key
    }

    pub fn add_function_node<F: Into<ComponentFn>>(&mut self, f: F) -> FunctionKey {
        let key = self.graph.add_function(f.into());
        self.state.init_func(key, &self.graph.functions[key].func);
        key
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

        self.state.transient.triggers = inputs.iter().copied()
            .map(|k| (k, TriggerState { recalculate: false }))
            .collect();
        self.state.transient.frontier.clear();

        let mut iteration = 0;
        while !self.state.transient.triggers.is_empty() || !self.state.transient.frontier.is_empty() {
            if iteration > RUN_ITER_LIMIT {
                for &key in self.state.transient.triggers.keys() {
                    index_mut(&mut self.state.values, &key).add_issue(ValueIssue::OscillationDetected);
                }
                break;
            }
            // 1. Update circuit state at start of cycle, save functions to waken in frontier
            for (node, TriggerState { recalculate }) in std::mem::take(&mut self.state.transient.triggers) {
                // Remove issues b/c of update
                self.state[node].clear_issues();
                
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
                        .map(|&Port { gate, index }| self.state[gate].ports[index]);
                    // Join:
                    let result = it.clone()
                        .reduce(BitArray::join)
                        .unwrap_or_else(|| BitArray::floating(self[node].len())); // if no inputs, make bits floating
                    // Short circuit check:
                    if BitArray::short_circuits(it) {
                        self.state[node].add_issue(ValueIssue::ShortCircuit);
                    }

                    propagate_update = self[node] != result;
                    self.state[node].set_value(result);
                }

                if propagate_update {
                    self.state.transient.frontier.extend({
                        self.graph[node].links.iter()
                            .filter(|&&Port { gate, index }| self.graph[gate].port_types[index].accepts_input())
                            .map(|&Port { gate, index: _ }| gate)
                    });
                }
            }
            // 2. For all functions to waken, apply function and save triggers for next cycle
            for gate_idx in std::mem::take(&mut self.state.transient.frontier) {
                let gate = &self.graph[gate_idx];
                let state = index_mut(&mut self.state.functions, &gate_idx);

                // Update inputs:
                let old_values = state.ports.clone();
                for (index, ((&port, port_value), port_type)) in std::iter::zip(&gate.links, &mut state.ports).zip(&gate.port_types).enumerate() {
                    if matches!(port_type, PortType::Output) { continue; }
                    // Only update inputs and inouts
                    let port_size = port_value.len();
                    let input = match port {
                        Some(n) => {
                            let node = index_mut(&mut self.state.values, &n);
                            let value = node.get_value();
                            let val_size = value.len();
                            match val_size == port_size {
                                true  => value,
                                false => {
                                    node.add_issue(ValueIssue::MismatchedBitsizes {
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
                
                for PortUpdate { index, value } in gate.func.run(&old_values, &state.ports) {
                    debug_assert!(self.graph[gate_idx].port_types[index].accepts_output(), "Input port cannot be updated");
                    if self.state[gate_idx].ports[index] != value {
                        self.state[gate_idx].ports[index] = value; // todo: assert right bitsize
                        
                        if let Some(sink_idx) = self.graph[gate_idx].links[index] {
                            self.state.transient.triggers.insert(sink_idx, TriggerState { recalculate: true });
                        }
                    }
                }
            }

            iteration += 1;
        }
    }

    pub fn get_issues(&self, key: ValueKey) -> &HashSet<ValueIssue> {
        self.state[key].get_issues()
    }
    pub fn try_set(&mut self, key: ValueKey, val: BitArray) -> Result<(), ()> {
        match self.state[key].set_value(val) {
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
        &self.state[index].value
    }
}
impl Index<FunctionKey> for Circuit {
    type Output = ComponentFn;

    fn index(&self, index: FunctionKey) -> &Self::Output {
        &self.graph[index].func
    }
}