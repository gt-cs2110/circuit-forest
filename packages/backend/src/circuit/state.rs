use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::ops::{Index, IndexMut};

use crate::bitarray::{bitarr, BitArray};
use crate::circuit::{CircuitGraph, FunctionKey, FunctionPort, ValueIssue, ValueKey};
use crate::node::{Component, ComponentFn, PortType, PortUpdate};

pub trait StateGetter {
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

#[derive(Default)]
pub struct CircuitState {
    pub(crate) values: HashMap<ValueKey, ValueState>,
    pub(crate) functions: HashMap<FunctionKey, FunctionState>,
    pub(crate) transient: TransientState
}
impl CircuitState {
    pub(crate) fn init_value(&mut self, key: ValueKey) {
        if let Entry::Vacant(e) = self.values.entry(key) {
            e.insert(ValueState::new(BitArray::new()));
        }
    }
    pub(crate) fn init_func(&mut self, key: FunctionKey, func: &ComponentFn) {
        if let Entry::Vacant(e) = self.functions.entry(key) {
            e.insert(FunctionState::initialize(func));
        }
    }

    fn init_from_graph(&mut self, graph: &CircuitGraph) {
        let mut state = CircuitState::default();
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
                    index_mut(&mut self.values, key).add_issue(ValueIssue::OscillationDetected);
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
                let state = index_mut(&mut self.functions, &gate_idx);

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
                        .map(|n| self.values[&n].get_value())
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

fn index_mut<'m, K: std::hash::Hash + Eq, V>(map: &'m mut HashMap<K, V>, index: &'_ K) -> &'m mut V {
    map.get_mut(index).unwrap()
}
impl Index<ValueKey> for CircuitState {
    type Output = ValueState;

    fn index(&self, index: ValueKey) -> &Self::Output {
        &self.values[&index]
    }
}
impl IndexMut<ValueKey> for CircuitState {
    fn index_mut(&mut self, index: ValueKey) -> &mut Self::Output {
        index_mut(&mut self.values, &index)
    }
}
impl Index<FunctionKey> for CircuitState {
    type Output = FunctionState;

    fn index(&self, index: FunctionKey) -> &Self::Output {
        &self.functions[&index]
    }
}
impl IndexMut<FunctionKey> for CircuitState {
    fn index_mut(&mut self, index: FunctionKey) -> &mut Self::Output {
        index_mut(&mut self.functions, &index)
    }
}

pub struct ValueState {
    pub(crate) value: BitArray,
    issues: HashSet<ValueIssue>
}
impl ValueState {
    pub fn new(value: BitArray) -> Self {
        Self { value, issues: HashSet::new() }
    }

    pub fn get_value(&self) -> BitArray {
        self.value
    }
    pub fn replace_value(&mut self, new_val: BitArray) -> Result<(), crate::bitarray::MismatchedBitsizes> {
        match self.value.is_empty() {
            true => {
                self.value = new_val;
                Ok(())
            },
            false => self.value.replace(new_val)
        }
    }

    pub fn get_issues(&self) -> &HashSet<ValueIssue> {
        &self.issues
    }
    pub fn add_issue(&mut self, issue: ValueIssue) {
        self.issues.insert(issue);
    }
    pub fn clear_issues(&mut self) {
        self.issues.clear()
    }
}
pub struct FunctionState {
    pub(crate) ports: Vec<BitArray>
}
impl FunctionState {
    pub fn initialize(func: &ComponentFn) -> Self {
        let mut ports: Vec<_> = func.ports().into_iter()
            .map(|props| bitarr![Z; props.bitsize])
            .collect();

        func.initialize(&mut ports);
        Self { ports }
    }

    pub fn get_port(&self, index: usize) -> BitArray {
        self.ports[index]
    }
    pub fn set_port(&mut self, index: usize, new_val: BitArray) -> Result<(), crate::bitarray::MismatchedBitsizes> {
        self.ports[index].replace(new_val)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct TriggerState {
    pub(crate) recalculate: bool
}
#[derive(Default)]
pub struct TransientState {
    pub(crate) triggers: HashMap<ValueKey, TriggerState>,
    pub(crate) frontier: HashSet<FunctionKey>
}
impl TransientState {
    pub fn resolved(&self) -> bool {
        self.triggers.is_empty() && self.frontier.is_empty()
    }
}