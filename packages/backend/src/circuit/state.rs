//! Module which contains data about circuit state.
//! 
//! This notably includes types which hold the state within a circuit, such as:
//! - [`CircuitState`]: The state in a circuit
//! - [`ValueState`]: The state of a value node (wire)
//! - [`FunctionState`]: The state of a function node

use std::collections::HashSet;
use std::ops::{Index, IndexMut};

use slotmap::{SecondaryMap, SparseSecondaryMap};
use slotmap::secondary::Entry;

use crate::bitarray::{bitarr, BitArray};
use crate::circuit::{CircuitGraphMap, CircuitKey, FunctionKey, FunctionPort, ValueIssue, ValueKey};
use crate::func::{Component, ComponentFn, PortType, PortUpdate, RunContext};

/// The state of the circuit.
/// 
/// This includes all wire values, all port values, and internal function state.
#[derive(Default, Debug)]
pub struct CircuitState {
    pub(crate) values: SecondaryMap<ValueKey, ValueState>,
    pub(crate) functions: SecondaryMap<FunctionKey, FunctionState>,
    pub(crate) transient: TransientState
}
impl CircuitState {
    /// Creates a new empty CircuitState.
    pub fn new() -> Self {
        Default::default()
    }
    
    /// Initializes a value node's state in this CircuitState.
    pub(crate) fn init_value(&mut self, key: ValueKey) {
        if let Some(Entry::Vacant(e)) = self.values.entry(key) {
            e.insert(ValueState::new(BitArray::new()));
        }
    }
    /// Initializes a function node's state in this CircuitState.
    pub(crate) fn init_func(&mut self, key: FunctionKey, func: &ComponentFn, graphs: &CircuitGraphMap) {
        if let Some(Entry::Vacant(e)) = self.functions.entry(key) {
            e.insert(FunctionState::new(func, graphs));
        }
    }

    /// Creates an initial [`CircuitState`] from a [`CircuitGraph`].
    pub(crate) fn init_from_graph(graphs: &CircuitGraphMap, key: CircuitKey) -> Self {
        let graph = &graphs[key];

        let mut state = Self::default();
        for k in graph.values.keys() {
            state.init_value(k);
        }
        for (k, f) in graph.functions.iter() {
            state.init_func(k, &f.func, graphs);
        }

        // All values tentatively need to be recomputed
        state.transient.values.extend({
            graph.values.keys()
                .map(|k| (k, PropagationState { recalculate: true }))
        });
        state.propagate(graphs, key);

        state
    }

    /// Gets the bit value of a [`ValueNode`].
    /// 
    /// [`ValueNode`]: crate::circuit::graph::ValueNode
    pub fn get_node_value(&self, k: ValueKey) -> BitArray {
        self[k].get_value()
    }
    /// Gets the bit value of a port attached to a [`FunctionNode`].
    /// 
    /// [`FunctionNode`]: crate::circuit::graph::FunctionNode
    pub fn get_port_value(&self, p: FunctionPort) -> BitArray {
        self[p.gate].get_port(p.index)
    }
    /// Gets all issues associated with a given [`ValueNode`].
    /// 
    /// [`ValueNode`]: crate::circuit::graph::ValueNode
    pub fn get_issues(&self, k: ValueKey) -> &HashSet<ValueIssue> {
        &self[k].issues
    }
    /// Removes a value node from CircuitState.
    /// 
    /// If this function is called, then `CircuitState::get_node_value`
    /// should NOT be called on this ValueKey.
    pub fn remove_node_value(&mut self, k: ValueKey) {
        self.values.remove(k);
        self.transient.values.remove(k);
    }
    /// Removes a value node from CircuitState.
    /// 
    /// If this function is called, then `CircuitState::get_node_value`
    /// should NOT be called on this ValueKey.
    pub fn remove_function_value(&mut self, k: FunctionKey) {
        self.functions.remove(k);
        self.transient.functions.remove(&k);
    }
    /// Signals that an update should be propagated from a value node.
    /// 
    /// If `recalculate` is true, this also recomputes the node's bit value
    /// before propagating.
    /// 
    pub(crate) fn add_transient(&mut self, k: ValueKey, recalculate: bool) {
        self.transient.values.insert(k, PropagationState { recalculate });
    }

    /// Pushes transient state, propagating any updates through
    /// (until the circuit stabilizes or an oscillation occurs).
    /// 
    /// This takes the graph to determine the relationship between nodes.
    pub fn propagate(&mut self, graphs: &CircuitGraphMap, key: CircuitKey) {
        let graph = &graphs[key];
        const RUN_ITER_LIMIT: usize = 10_000;

        let mut iteration = 0;
        while !self.transient.resolved() {
            if iteration > RUN_ITER_LIMIT {
                for key in self.transient.values.keys() {
                    self.values[key].add_issue(ValueIssue::OscillationDetected);
                }
                break;
            }
            // 1. Update circuit state at start of cycle, save functions to waken in frontier
            for (node, PropagationState { recalculate }) in std::mem::take(&mut self.transient.values) {
                // Remove issues b/c of update
                self[node].clear_issues();
                
                // Iterator of all port values
                // We join them using a join algorithm.
                // If the value changes after this join, 
                // then we know we should propagate the update to the function.

                let mut propagate_update = true;
                if recalculate {
                    let result = match graph[node].bitsize {
                        Some(s) => {
                            // Get all port values feeding into value
                            let feed_it = graph[node].links.iter()
                                .filter(|p| graph[p.gate].port_props[p.index].ty.accepts_output())
                                .map(|&p| self.get_port_value(p));
                            // Find value and short circuit status
                            let (result, occupied) = feed_it.fold(
                                (bitarr![Z; s], Some(0)),
                                |(array, m_occupied), current| (
                                    array.join(current),
                                    m_occupied.and_then(|occupied| current.short_circuits(occupied))
                                )
                            );

                            if occupied.is_none() {
                                self[node].add_issue(ValueIssue::ShortCircuit);
                            }
                            result
                        },
                        None => {
                            self[node].add_issue(ValueIssue::MismatchedBitsizes);
                            BitArray::new()
                        }
                    };

                    propagate_update = self.get_node_value(node) != result;
                    self[node].value = result;
                }

                if propagate_update {
                    self.transient.functions.extend({
                        graph[node].links.iter()
                            .filter(|p| graph[p.gate].port_props[p.index].ty.accepts_input())
                            .map(|p| p.gate)
                    });
                }
            }
            // 2. For all functions to waken, apply function and save triggers for next cycle
            for gate_idx in std::mem::take(&mut self.transient.functions) {
                let gate = &graph[gate_idx];
                let state = &mut self.functions[gate_idx];

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
                        .filter(|&n| graph[n].bitsize == Some(props.bitsize))
                        .map(|n| self.values[n].get_value())
                        .unwrap_or_else(|| bitarr![Z; props.bitsize]);
                }
                
                let ctx = RunContext {
                    graphs,
                    old_ports: &old_values,
                    new_ports: &state.ports,
                    inner_state: state.inner.as_mut()
                };
                for PortUpdate { index, value } in gate.func.run(ctx) {
                    // Push outputs:
                    debug_assert!(graph[gate_idx].port_props[index].ty.accepts_output(), "Input port cannot be updated");
                    debug_assert_eq!(graph[gate_idx].port_props[index].bitsize, value.len(), "Expected value to have matching bitsize");
                    if self[gate_idx].ports[index] != value {
                        self[gate_idx].ports[index] = value;
                        
                        if let Some(sink_idx) = graph[gate_idx].links[index] {
                            self.add_transient(sink_idx, true);
                        }
                    }
                }
            }

            iteration += 1;
        }
    }
}

impl Index<ValueKey> for CircuitState {
    type Output = ValueState;

    fn index(&self, index: ValueKey) -> &Self::Output {
        &self.values[index]
    }
}
impl IndexMut<ValueKey> for CircuitState {
    fn index_mut(&mut self, index: ValueKey) -> &mut Self::Output {
        &mut self.values[index]
    }
}
impl Index<FunctionKey> for CircuitState {
    type Output = FunctionState;

    fn index(&self, index: FunctionKey) -> &Self::Output {
        &self.functions[index]
    }
}
impl IndexMut<FunctionKey> for CircuitState {
    fn index_mut(&mut self, index: FunctionKey) -> &mut Self::Output {
        &mut self.functions[index]
    }
}

/// The state of a [`ValueNode`].
/// 
/// [`ValueNode`]: crate::circuit::graph::ValueNode
#[derive(Debug)]
pub struct ValueState {
    pub(crate) value: BitArray,
    issues: HashSet<ValueIssue>
}
impl ValueState {
    /// Creates a new ValueState from the given [`BitArray`].
    pub fn new(value: BitArray) -> Self {
        Self { value, issues: HashSet::new() }
    }

    /// Gets the bit value in the state.
    pub fn get_value(&self) -> BitArray {
        self.value
    }
    /// Sets the bit value in the state.
    /// 
    /// This raises an error if the bitsize of the new value
    /// doesn't match the bitsize of the current.
    /// 
    /// Notably, if the current bit value is empty (size 0),
    /// then this always succeeds and does not raise an error.
    /// 
    /// This also does **not** propagate the change through the circuit this ValueState is associated with.
    pub(crate) fn replace_value(&mut self, new_val: BitArray) -> Result<(), crate::bitarray::MismatchedBitsizes> {
        match self.value.is_empty() {
            true => {
                self.value = new_val;
                Ok(())
            },
            false => self.value.replace(new_val)
        }
    }


    /// Gets any issues in the value node.
    pub fn get_issues(&self) -> &HashSet<ValueIssue> {
        &self.issues
    }
    /// Adds an issue to the value node's issue list.
    pub fn add_issue(&mut self, issue: ValueIssue) {
        self.issues.insert(issue);
    }
    /// Removes all issues from the value node's issue list.
    pub fn clear_issues(&mut self) {
        self.issues.clear()
    }
}

/// Internal state for a function node.
/// 
/// This isn't needed if all the information can be pulled from port data,
/// but is useful for when larger state is needed (e.g., subcircuits, RAM).
#[derive(Debug)]
pub enum InnerFunctionState {
    /// Subcircuit data.
    Subcircuit(CircuitState),
}


/// The state of a [`FunctionNode`].
/// 
/// [`FunctionNode`]: crate::circuit::graph::FunctionNode
#[derive(Debug)]
pub struct FunctionState {
    pub(crate) ports: Vec<BitArray>,
    pub(crate) inner: Option<InnerFunctionState>
}
impl FunctionState {
    /// Creates a new initial function state for the specified `func`.
    pub fn new(func: &ComponentFn, graphs: &CircuitGraphMap) -> Self {
        let mut ports: Vec<_> = func.ports(graphs).into_iter()
            .map(|props| bitarr![Z; props.bitsize])
            .collect();
        
        func.initialize_port_state(&mut ports);
        let inner = func.initialize_inner_state(graphs);
        Self { ports, inner }
    }

    /// Gets the bit value of a port.
    pub fn get_port(&self, index: usize) -> BitArray {
        self.ports[index]
    }
    /// Sets the bit value of a port.
    /// 
    /// This raises an error if the bitsize of the new value
    /// doesn't match the bitsize of the current.
    /// 
    /// Notably, if the current bit value is empty (size 0),
    /// then this always succeeds and does not raise an error.
    /// 
    /// This also does **not** propagate the change through the circuit this FunctionState is associated with.
    pub(crate) fn replace_port(&mut self, index: usize, new_val: BitArray) -> Result<(), crate::bitarray::MismatchedBitsizes> {
        self.ports[index].replace(new_val)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PropagationState {
    /// If true, then this value should be recalculated before being propagated.
    pub(crate) recalculate: bool
}

/// Temporary propagation state.
#[derive(Default, Debug)]
pub(crate) struct TransientState {
    /// Determines which values need to be propagated.
    pub(crate) values: SparseSecondaryMap<ValueKey, PropagationState>,
    /// Determines which functions need to recalculate their state and propagate their outputs.
    pub(crate) functions: HashSet<FunctionKey>
}
impl TransientState {
    pub fn resolved(&self) -> bool {
        self.values.is_empty() && self.functions.is_empty()
    }
}