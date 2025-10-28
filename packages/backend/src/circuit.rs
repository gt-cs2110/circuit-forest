//! Circuit module used to create circuits/sub-circuits.

mod state;

use std::collections::HashSet;
use std::ops::{Index, IndexMut};

use slotmap::{new_key_type, SlotMap};

use crate::bitarray::{bitarr, BitArray};
use crate::circuit::state::{index_mut, CircuitState, TriggerState, ValueState};
use crate::node::{Component, ComponentFn, PortProperties, PortType, PortUpdate};


new_key_type! {
    /// Key type for maps to values.
    pub struct ValueKey;
    /// Key type for maps to functions.
    pub struct FunctionKey;
}

/// A struct which identifies a port (from its function node and port index).
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug)]
pub struct FunctionPort {
    gate: FunctionKey,
    index: usize
}

/// Issues which can occur to a value node.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueIssue {
    /// Represents a collision of bit values (short circuit).
    ShortCircuit,

    /// Represents a connection with two different bitsizes.
    MismatchedBitsizes,

    /// Represents a connection whose value is unstable.
    OscillationDetected
}

/// A node which represents a wire (which can hold some value).
/// 
/// This node may only connect to function node ports.
#[derive(Default)]
pub struct ValueNode {
    /// The current bitsize.
    /// 
    /// This is used to check for bitsize mismatches.
    /// This is None if there is nothing or mismatched bitsizes
    /// connecting to or from this node.
    bitsize: Option<u8>,

    /// Ports this node is connected to.
    links: HashSet<FunctionPort>
}
impl ValueNode {
    /// Creates a new value node without a specified bitsize.
    pub fn new() -> Self {
        Default::default()
    }
}

/// A node which represents a function or component.
/// 
/// This node has ports which may only connect to value nodes.
pub struct FunctionNode {
    /// The actual function that is applied.
    func: ComponentFn,
    /// The properties of this function's ports.
    port_props: Vec<PortProperties>,
    // Value nodes each port is connected to (if connected to a port).
    links: Vec<Option<ValueKey>>,
}
impl FunctionNode {
    /// Creates a new function node with the specified component function type.
    pub fn new(func: ComponentFn) -> Self {
        let port_props = func.ports();
        let links = vec![None; port_props.len()];
        
        Self { func, links, port_props }
    }
}

/// A circuit structure.
#[derive(Default)]
pub struct CircuitGraph {
    /// All value nodes of the circuit.
    values: SlotMap<ValueKey, ValueNode>,
    /// All function nodes of the circuit.
    functions: SlotMap<FunctionKey, FunctionNode>,
}
impl CircuitGraph {
    /// Adds a new value node to the graph and returns its index.
    pub fn add_value(&mut self) -> ValueKey {
        self.values.insert(ValueNode::new())
    }

    /// Adds a new function node to the graph (using the provided component function)
    /// and returns its index.
    pub fn add_function(&mut self, func: ComponentFn) -> FunctionKey {
        self.functions.insert(FunctionNode::new(func))
    }

    /// When connecting another port to the value node,
    /// update the value node's current bitsize with
    /// the new connection (with specified bitsize).
    fn attach_bitsize(&mut self, key: ValueKey, bitsize: u8) {
        self.values[key].bitsize = match self.values[key].links.len() {
            2.. => self.values[key].bitsize.filter(|&s| s == bitsize),
            _ => Some(bitsize)
        }
    }
    /// Recalculate the bitsize, based on all of the node's links.
    fn recalculate_bitsize(&mut self, key: ValueKey) {
        // None if no elements or conflicting elements
        // Some(n) if there's exactly one
        fn reduce(mut it: impl Iterator<Item=u8>) -> Option<u8> {
            let bitsize = it.next()?;
            it.all(|s| bitsize == s).then_some(bitsize)
        }

        // Iterator of all function ports connecting to & from value
        let bitsize_it = self.values[key].links.iter()
            .map(|p| self.functions[p.gate].port_props[p.index].bitsize);
        self.values[key].bitsize = reduce(bitsize_it);
    }

    /// Connect a value node to a function port.
    pub fn connect(&mut self, wire: ValueKey, port: FunctionPort) {
        self.disconnect(port);
        self.functions[port.gate].links[port.index].replace(wire);

        self.values[wire].links.insert(port);
        self.attach_bitsize(wire, self.functions[port.gate].port_props[port.index].bitsize);
    }
    /// Disconnect an associated value node (if one exists) from a function port.
    pub fn disconnect(&mut self, port: FunctionPort) {
        let old_port = self.functions[port.gate].links[port.index];
        // If there was something there, remove it from the other side:
        if let Some(node) = old_port {
            let result = self.values[node].links.remove(&port);
            debug_assert!(result, "Gate should've been removed from assigned value node");

            self.recalculate_bitsize(node);
        }
    }
    /// Clears all ports from a function node.
    pub fn clear_edges(&mut self, gate: FunctionKey) {
        for index in 0..self.functions[gate].links.len() {
            let Some(node) = self.functions[gate].links[index].take() else { continue };

            let result = self.values[node].links.remove(&FunctionPort { gate, index });
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

/// A circuit, which includes its structure ([`CircuitGraph`]) and its state ([`CircuitState`]).
#[derive(Default)]
pub struct Circuit {
    graph: CircuitGraph,
    state: CircuitState
}

impl Circuit {
    /// Constructs an empty Circuit.
    pub fn new() -> Self {
        Default::default()
    }
    
    /// Adds an input function node and value node (a wire connecting from the input)
    /// using the passed value.
    pub fn add_input(&mut self, arr: BitArray) -> ValueKey {
        let func = self.add_function_node(crate::node::Input::new(arr));
        let value = self.add_value_node(arr.len());

        self.connect_all(func, &[value]);

        value
    }
    
    /// Create a value node (essentially a wire) with the specified bitsize.
    pub fn add_value_node(&mut self, bitsize: u8) -> ValueKey {
        let key = self.graph.add_value();
        self.state.values.insert(key, ValueState::new(bitarr![Z; bitsize]));
        key
    }

    /// Create a function node with the passed component function.
    pub fn add_function_node<F: Into<ComponentFn>>(&mut self, f: F) -> FunctionKey {
        let key = self.graph.add_function(f.into());
        self.state.init_func(key, &self.graph.functions[key].func);
        key
    }

    /// Connect a wire to a port in the Circuit's graph.
    pub fn connect_one(&mut self, wire: ValueKey, port: FunctionPort) {
        self.graph.connect(wire, port);
        self.state.transient.triggers.insert(wire, TriggerState { recalculate: true });
        self.propagate();
    }

    /// Clear the function node of connections and connect all of the passed ports to it.
    pub fn connect_all(&mut self, gate: FunctionKey, ports: &[ValueKey]) {
        self.graph.clear_edges(gate);
        ports.iter().copied()
            .enumerate()
            .for_each(|(index, wire)| {
                self.graph.connect(wire, FunctionPort { gate, index });
                self.state.transient.triggers.insert(wire, TriggerState { recalculate: true });
            });
        self.propagate();
    }

    /// Propagates an update through the circuit
    /// (until the circuit stabilizes or an oscillation occurs).
    /// 
    /// The provided `input` argument indicates which value nodes were updated.
    pub fn run(&mut self, inputs: &[ValueKey]) {
        self.state.transient.triggers = inputs.iter().copied()
            .map(|k| (k, TriggerState { recalculate: false }))
            .collect();
        self.state.transient.frontier.clear();

        self.propagate();
    }

    /// Pushes transient state, propagating any updates through
    /// (until the circuit stabilizes or an oscillation occurs).
    pub fn propagate(&mut self) {
        const RUN_ITER_LIMIT: usize = 10_000;

        let mut iteration = 0;
        while !self.state.transient.resolved() {
            if iteration > RUN_ITER_LIMIT {
                for key in self.state.transient.triggers.keys() {
                    index_mut(&mut self.state.values, key).add_issue(ValueIssue::OscillationDetected);
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
                    let result = match self.graph[node].bitsize {
                        Some(s) => {
                            // Get all port values feeding into value
                            let feed_it = self.graph[node].links.iter()
                                .filter(|p| self.graph[p.gate].port_props[p.index].ty.accepts_output())
                                .map(|&p| self.state.value(p));
                            // Find value and short circuit status
                            let (result, occupied) = feed_it.fold(
                                (bitarr![Z; s], Some(0)),
                                |(array, m_occupied), current| (
                                    array.join(current),
                                    m_occupied.and_then(|occupied| current.short_circuits(occupied))
                                )
                            );

                            if occupied.is_none() {
                                self.state[node].add_issue(ValueIssue::ShortCircuit);
                            }
                            result
                        },
                        None => {
                            self.state[node].add_issue(ValueIssue::MismatchedBitsizes);
                            BitArray::new()
                        }
                    };

                    propagate_update = self.state.value(node) != result;
                    self.state[node].value = result;
                }

                if propagate_update {
                    self.state.transient.frontier.extend({
                        self.graph[node].links.iter()
                            .filter(|p| self.graph[p.gate].port_props[p.index].ty.accepts_input())
                            .map(|p| p.gate)
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
                    .zip(&mut state.ports);
                for ((&port, props), port_value) in it {
                    if matches!(props.ty, PortType::Output) { continue; }
                    // Update inputs and inouts
                    // Replace any disconnected ports and mismatched bitsizes with floating
                    *port_value = port
                        .filter(|&n| self.graph[n].bitsize == Some(props.bitsize))
                        .map(|n| self.state.values[&n].get_value())
                        .unwrap_or_else(|| bitarr![Z; props.bitsize]);
                }
                
                for PortUpdate { index, value } in gate.func.run(&old_values, &state.ports) {
                    // Push outputs:
                    debug_assert!(self.graph[gate_idx].port_props[index].ty.accepts_output(), "Input port cannot be updated");
                    debug_assert_eq!(self.graph[gate_idx].port_props[index].bitsize, value.len(), "Expected value to have matching bitsize");
                    if self.state[gate_idx].ports[index] != value {
                        self.state[gate_idx].ports[index] = value;
                        
                        if let Some(sink_idx) = self.graph[gate_idx].links[index] {
                            self.state.transient.triggers.insert(sink_idx, TriggerState { recalculate: true });
                        }
                    }
                }
            }

            iteration += 1;
        }
    }

    /// Gets current circuit state.
    pub fn state(&self) -> &CircuitState {
        &self.state
    }

    /// Updates a ValueNode with the specified value, raising `Err` if bitsizes do not match.
    pub fn try_set(&mut self, key: ValueKey, val: BitArray) -> Result<(), crate::bitarray::MismatchedBitsizes> {
        self.state[key].replace_value(val)
    }

    /// Updates a ValueNode with the specified value.
    /// 
    /// This function panics if the bitsize doesn't match.
    pub fn set(&mut self, key: ValueKey, val: BitArray) {
        self.try_set(key, val)
            .expect("Tried to set value with wrong bitsize")
    }
}

#[test]
fn test_try_set_and_set() {
    let mut circuit = Circuit::new(); // create empty circuit
    
    let value = bitarr![0; 8]; // Create a 8 bit BitArray 
    let key = circuit.add_value_node(8); //add a ValueKey of 8 bits

    // try_set should succeed
    assert!(circuit.try_set(key, value).is_ok());

    // set should succeed 
    let value = bitarr![1; 8]; // Create a 8 bit BitArray of high 
    circuit.set(key, value);

    // intentionally wrong bitsize should fail
    let wrong_value = bitarr![0; 16]; // 16 bits instead of 8
    assert!(circuit.try_set(key, wrong_value).is_err());
}
