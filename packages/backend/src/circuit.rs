mod state;

use std::collections::HashSet;
use std::ops::{Index, IndexMut};

use slotmap::{new_key_type, SlotMap};

use crate::bitarray::BitArray;
use crate::circuit::state::{index_mut, CircuitState, TriggerState, ValueState};
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

#[derive(Default)]
struct ValueNode {
    default_bitsize: Option<u8>,
    bitsize: Option<u8>,
    links: HashSet<Port>
}
impl ValueNode {
    fn new() -> Self {
        Default::default()
    }
    fn with_bitsize(bitsize: u8) -> Self {
        Self {
            default_bitsize: Some(bitsize),
            bitsize: Some(bitsize),
            links: HashSet::new(),
        }
    }
}
struct FunctionNode {
    func: ComponentFn,
    port_props: Vec<PortProperties>,
    // connections
    links: Vec<Option<ValueKey>>,
}
impl FunctionNode {
    fn new(func: ComponentFn) -> Self {
        let port_props = func.ports();
        let links = vec![None; port_props.len()];
        
        Self { func, links, port_props }
    }
}
#[derive(Default)]
struct CircuitGraph {
    values: SlotMap<ValueKey, ValueNode>,
    functions: SlotMap<FunctionKey, FunctionNode>,
}
impl CircuitGraph {
    pub fn add_value(&mut self) -> ValueKey {
        self.values.insert(ValueNode::new())
    }
    pub fn add_function(&mut self, func: ComponentFn) -> FunctionKey {
        self.functions.insert(FunctionNode::new(func))
    }

    fn attach_bitsize(&mut self, key: ValueKey, bitsize: u8) {
        self.values[key].bitsize = self.values[key].bitsize.filter(|&s| s == bitsize);
    }
    fn recalculate_bitsize(&mut self, key: ValueKey) {
        // None if no elements or conflicting elements
        // Some(n) if there's exactly one
        fn reduce(init: Option<u8>, mut it: impl Iterator<Item=u8>) -> Option<u8> {
            let bitsize = init.or_else(|| it.next())?;
            it.all(|s| bitsize == s).then_some(bitsize)
        }

        self.values[key].bitsize = reduce(
            self.values[key].default_bitsize, 
            self.values[key].links.iter().map(|p| {
                self.functions[p.gate].port_props[p.index].bitsize
            })
        );
    }

    pub fn connect(&mut self, wire: ValueKey, port: Port) {
        self.disconnect(port);
        self.functions[port.gate].links[port.index].replace(wire);

        self.values[wire].links.insert(port);
        self.attach_bitsize(wire, self.functions[port.gate].port_props[port.index].bitsize);
    }
    pub fn disconnect(&mut self, port: Port) {
        let old_port = self.functions[port.gate].links[port.index];
        // If there was something there, remove it from the other side:
        if let Some(node) = old_port {
            let result = self.values[node].links.remove(&port);
            debug_assert!(result, "Gate should've been removed from assigned value node");

            self.recalculate_bitsize(node);
        }
    }
    pub fn clear_edges(&mut self, gate: FunctionKey) {
        for index in 0..self.functions[gate].links.len() {
            let Some(node) = self.functions[gate].links[index].take() else { continue };

            let result = self.values[node].links.remove(&Port { gate, index });
            debug_assert!(result, "Gate should've been removed from assigned value node");
            self.recalculate_bitsize(node);
        }
    }
}
impl Index<ValueKey> for CircuitGraph {
    type Output = ValueNode;

    fn index(&self, index: ValueKey) -> &Self::Output {
        &self.values[index]
    }
}
impl IndexMut<ValueKey> for CircuitGraph {
    fn index_mut(&mut self, index: ValueKey) -> &mut Self::Output {
        &mut self.values[index]
    }
}
impl Index<FunctionKey> for CircuitGraph {
    type Output = FunctionNode;

    fn index(&self, index: FunctionKey) -> &Self::Output {
        &self.functions[index]
    }
}
impl IndexMut<FunctionKey> for CircuitGraph {
    fn index_mut(&mut self, index: FunctionKey) -> &mut Self::Output {
        &mut self.functions[index]
    }
}
#[derive(Default)]
pub struct Circuit {
    graph: CircuitGraph,
    state: CircuitState
}

impl Circuit {
    pub fn new() -> Self {
        Default::default()
    }
    
    fn add_empty_value_node(&mut self) -> ValueKey {
        let key = self.graph.add_value();
        self.state.init_value(key);
        key
    }
    pub fn add_value_node(&mut self, arr: BitArray) -> ValueKey {
        let key = self.add_empty_value_node();
        self.state.values.insert(key, ValueState::new(arr));
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
        while !self.state.transient.resolved() {
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
                        .filter(|&&Port { gate, index }| self.graph[gate].port_props[index].ty.accepts_output())
                        .map(|&Port { gate, index }| self.state[gate].ports[index]);
                    // Join:
                    let result = it.clone()
                        .fold(BitArray::floating(self[node].len()), BitArray::join);
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
                            .filter(|&&Port { gate, index }| self.graph[gate].port_props[index].ty.accepts_input())
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
                let it = gate.links.iter()
                    .zip(&gate.port_props)
                    .zip(&mut state.ports)
                    .enumerate();
                for (index, ((&port, props), port_value)) in it {
                    if matches!(props.ty, PortType::Output) { continue; }
                    // Only update inputs and inouts
                    let input = port.and_then(|n| {
                        let node = index_mut(&mut self.state.values, &n);
                        let node_value = node.get_value();
                        match node_value.len() == port_value.len() {
                            true  => Some(node_value),
                            false => {
                                node.add_issue(ValueIssue::MismatchedBitsizes {
                                    val_size: node_value.len(),
                                    port_size: port_value.len(),
                                    port: Port { gate: gate_idx, index },
                                    is_input: true
                                });
                                None
                            }
                        }
                    });
                    
                    *port_value = input.unwrap_or_else(|| BitArray::floating(port_value.len()));
                }
                
                for PortUpdate { index, value } in gate.func.run(&old_values, &state.ports) {
                    debug_assert!(self.graph[gate_idx].port_props[index].ty.accepts_output(), "Input port cannot be updated");
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