use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::ops::{Index, IndexMut};

use crate::bitarray::BitArray;
use crate::circuit::{CircuitGraph, FunctionKey, ValueIssue, ValueKey};
use crate::node::{Component, ComponentFn};

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

    pub(crate) fn init_from_graph(&mut self, graph: &CircuitGraph) {
        let mut state = CircuitState::default();
        for k in graph.values.keys() {
            state.init_value(k);
        }
        for (k, f) in graph.functions.iter() {
            state.init_func(k, &f.func);
        }
    }
}

pub(crate) fn index_mut<'m, K: std::hash::Hash + Eq, V>(map: &'m mut HashMap<K, V>, index: &'_ K) -> &'m mut V {
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
    #[must_use]
    pub fn set_value(&mut self, new_val: BitArray) -> bool {
        let success = self.value.len() == new_val.len();
        if success {
            self.value = new_val;
        }
        success
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
            .map(|props| BitArray::floating(props.bitsize))
            .collect();

        func.initialize(&mut ports);
        Self { ports }
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