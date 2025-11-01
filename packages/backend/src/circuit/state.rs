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
use crate::circuit::{CircuitGraph, FunctionKey, FunctionPort, ValueIssue, ValueKey};
use crate::func::{Component, ComponentFn, PortType, PortUpdate};

/// Trait which allows reading a [`BitArray`] value from [`CircuitState`].
pub trait StateGetter {
    /// Gets the bit array value from the circuit state.
    /// 
    /// Panics if not in state.
    fn get_value(&self, state: &CircuitState) -> BitArray;
}
impl StateGetter for ValueKey {
    fn get_value(&self, state: &CircuitState) -> BitArray {
        state[*self].get_value()
    }
}
impl StateGetter for FunctionPort {
    fn get_value(&self, state: &CircuitState) -> BitArray {
        state[self.gate].ports[self.index]
    }
}
impl<K: StateGetter> StateGetter for &K {
    fn get_value(&self, state: &CircuitState) -> BitArray {
        (*self).get_value(state)
    }
}

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
    pub(crate) fn init_func(&mut self, key: FunctionKey, func: &ComponentFn) {
        if let Some(Entry::Vacant(e)) = self.functions.entry(key) {
            e.insert(FunctionState::new(func));
        }
    }

    /// Creates an initial [`CircuitState`] from a [`CircuitGraph`].
    fn init_from_graph(graph: &CircuitGraph) -> Self {
        let mut state = Self::default();
        for k in graph.values.keys() {
            state.init_value(k);
        }
        for (k, f) in graph.functions.iter() {
            state.init_func(k, &f.func);
        }

        // All values tentatively need to be recomputed
        state.transient.triggers.extend({
            graph.values.keys()
                .map(|k| (k, TriggerState { recalculate: true }))
        });
        state.propagate(graph);

        state
    }

    pub(crate) fn value<K: StateGetter>(&self, k: K) -> BitArray {
        K::get_value(&k, self)
    }
    pub(crate) fn issues(&self, k: ValueKey) -> &HashSet<ValueIssue> {
        &self[k].issues
    }

    /// Pushes transient state, propagating any updates through
    /// (until the circuit stabilizes or an oscillation occurs).
    /// 
    /// This takes the graph to determine the relationship between nodes.
    pub fn propagate(&mut self, graph: &CircuitGraph) {
        const RUN_ITER_LIMIT: usize = 10_000;

        let mut iteration = 0;
        while !self.transient.resolved() {
            if iteration > RUN_ITER_LIMIT {
                for key in self.transient.triggers.keys() {
                    self.values[key].add_issue(ValueIssue::OscillationDetected);
                }
                break;
            }
            // 1. Update circuit state at start of cycle, save functions to waken in frontier
            for (node, TriggerState { recalculate }) in std::mem::take(&mut self.transient.triggers) {
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
                                .map(|&p| self.value(p));
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

                    propagate_update = self.value(node) != result;
                    self[node].value = result;
                }

                if propagate_update {
                    self.transient.frontier.extend({
                        graph[node].links.iter()
                            .filter(|p| graph[p.gate].port_props[p.index].ty.accepts_input())
                            .map(|p| p.gate)
                    });
                }
            }
            // 2. For all functions to waken, apply function and save triggers for next cycle
            for gate_idx in std::mem::take(&mut self.transient.frontier) {
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
                
                for PortUpdate { index, value } in gate.func.run(&old_values, &state.ports) {
                    // Push outputs:
                    debug_assert!(graph[gate_idx].port_props[index].ty.accepts_output(), "Input port cannot be updated");
                    debug_assert_eq!(graph[gate_idx].port_props[index].bitsize, value.len(), "Expected value to have matching bitsize");
                    if self[gate_idx].ports[index] != value {
                        self[gate_idx].ports[index] = value;
                        
                        if let Some(sink_idx) = graph[gate_idx].links[index] {
                            self.transient.triggers.insert(sink_idx, TriggerState { recalculate: true });
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
    pub fn replace_value(&mut self, new_val: BitArray) -> Result<(), crate::bitarray::MismatchedBitsizes> {
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

/// The state of a [`FunctionNode`].
/// 
/// [`FunctionNode`]: crate::circuit::graph::FunctionNode
#[derive(Debug)]
pub struct FunctionState {
    pub(crate) ports: Vec<BitArray>
}
impl FunctionState {
    /// Creates a new initial function state for the specified `func`.
    pub fn new(func: &ComponentFn) -> Self {
        let mut ports: Vec<_> = func.ports().into_iter()
            .map(|props| bitarr![Z; props.bitsize])
            .collect();

        func.initialize(&mut ports);
        Self { ports }
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
    pub fn set_port(&mut self, index: usize, new_val: BitArray) -> Result<(), crate::bitarray::MismatchedBitsizes> {
        self.ports[index].replace(new_val)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TriggerState {
    pub(crate) recalculate: bool
}
#[derive(Default, Debug)]
pub(crate) struct TransientState {
    pub(crate) triggers: SparseSecondaryMap<ValueKey, TriggerState>,
    pub(crate) frontier: HashSet<FunctionKey>
}
impl TransientState {
    pub fn resolved(&self) -> bool {
        self.triggers.is_empty() && self.frontier.is_empty()
    }
}