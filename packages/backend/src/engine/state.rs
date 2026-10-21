//! Module which contains data about circuit state.
//! 
//! This notably includes types which hold the state within a circuit, such as:
//! - [`CircuitState`]: The state in a circuit
//! - [`ValueState`]: The state of a value node (wire)
//! - [`FunctionState`]: The state of a function node

use std::collections::{hash_map, HashMap, HashSet};
use std::ops::{Index, IndexMut};

use slotmap::{SecondaryMap, SparseSecondaryMap};
use slotmap::secondary::Entry;

use crate::bitarray::{bitarr, BitArray, BitState};
use crate::engine::{CircuitGraph, CircuitGraphMap, CircuitKey, FunctionKey, FunctionPort, ValueKey};
use crate::engine::func::{Component, ComponentFn, PortBit, PortType, PortUpdate, RunContext};

/// Issues which can occur to a value node.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, strum::Display)]
pub enum ValueIssue {
    /// Represents a collision of bit values (short circuit).
    ShortCircuit,

    /// Represents a connection with two different bitsizes.
    MismatchedBitsizes,

    /// Represents a connection whose value is unstable.
    OscillationDetected
}

/// The state of the circuit.
/// 
/// This includes all wire values, all port values, and internal function state.
#[derive(Default)]
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

        // All values need to be recomputed.
        state.transient.values.extend({
            graph.values.keys()
                .map(|k| (k, PropagationState { recalculate: RecalcState::RecalcPass }))
        });
        state.propagate(graphs, key);

        state
    }

    /// Gets the bit value of a [`ValueNode`].
    /// 
    /// [`ValueNode`]: super::graph::ValueNode
    pub fn get_node_value(&self, k: ValueKey) -> BitArray {
        self[k].get_value()
    }
    /// Gets the bit value of a port attached to a [`FunctionNode`].
    /// 
    /// [`FunctionNode`]: super::graph::FunctionNode
    pub fn get_port_value(&self, p: FunctionPort) -> BitArray {
        self[p.gate].get_port(p.index)
    }
    /// Gets all issues associated with a given [`ValueNode`].
    /// 
    /// [`ValueNode`]: super::graph::ValueNode
    pub fn get_issues(&self, k: ValueKey) -> &HashSet<ValueIssue> {
        &self[k].issues
    }
    /// Removes a value node from CircuitState.
    /// 
    /// If this function is called, then [`CircuitState::get_node_value`]
    /// should NOT be called on this ValueKey.
    pub fn remove_node_value(&mut self, k: ValueKey) {
        self.values.remove(k);
        self.transient.values.remove(k);
    }
    /// Removes a value node from CircuitState.
    /// 
    /// If this function is called, then any queries requiring
    /// this [`FunctionKey`] should NOT be called.
    pub fn remove_function_value(&mut self, k: FunctionKey) {
        self.functions.remove(k);
        self.transient.functions.remove(&k);
    }

    /// Pushes transient state, propagating any updates through
    /// (until the circuit stabilizes or an oscillation occurs).
    /// 
    /// This takes the graph to determine the relationship between nodes.
    /// 
    /// This returns whether the propagation completes in full and does not result in oscillation.
    pub fn propagate(&mut self, graphs: &CircuitGraphMap, key: CircuitKey) -> bool {
        let graph = &graphs[key];
        const RUN_ITER_LIMIT: usize = 10_000;

        let mut iteration = 0;
        while !self.transient.resolved() {
            if iteration > RUN_ITER_LIMIT {
                for key in self.transient.values.keys() {
                    self.values[key].add_issue(ValueIssue::OscillationDetected);
                }
                return false;
            }
            // 1. Update circuit state at start of cycle, save functions to waken in frontier
            let batch = std::mem::take(&mut self.transient.values);
            let mut net_resolved = HashSet::new();
            for (node, &PropagationState { recalculate }) in batch.iter() {
                if net_resolved.contains(&node) { continue; }
                
                let updated = match graph[node].bitsize.is_some() && graph.has_passive_link(node) {
                    true => self.resolve_net(graph, node, &batch, &mut net_resolved),
                    false => match self.resolve_value(graph, node, recalculate) {
                        true => vec![node],
                        false => vec![]
                    }
                };

                for node in updated {
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
                            self.transient.insert(sink_idx, RecalcState::Recalc);
                        }
                    }
                }
            }

            iteration += 1;
        }
        true
    }

    /// Recalculates a value node which is not connected to any passive ports.
    ///
    /// This returns whether the update should propagate to the node's readers.
    fn resolve_value(&mut self, graph: &CircuitGraph, node: ValueKey, recalculate: RecalcState) -> bool {
        // Remove issues b/c of update
        self[node].clear_issues();
        if matches!(recalculate, RecalcState::Pass) { return true; }

        // Iterator of all port values
        // We join them using a join algorithm.
        // If the value changes after this join,
        // then we know we should propagate the update to the function.

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

        let changed = self.get_node_value(node) != result;
        self[node].value = result;
        matches!(recalculate, RecalcState::RecalcPass) || changed
    }

    /// Recalculates `start` and all value nodes connected to it through passive ports (e.g., splitters).
    ///
    /// Value node bits are grouped into nets, each net is resolved at once from all drivers across all of its value nodes.
    ///
    /// This returns the value nodes whose updates should propagate to their readers
    /// and extends `resolved` with all recalculated value nodes.
    fn resolve_net(
        &mut self,
        graph: &CircuitGraph,
        start: ValueKey,
        batch: &SparseSecondaryMap<ValueKey, PropagationState>,
        resolved: &mut HashSet<ValueKey>
    ) -> Vec<ValueKey> {
        // Find all value nodes connected through passive ports,
        // assigning each bit of each value node an index which will be used in Union-Find.
        // `offsets` goes from a ValueKey to where the bits of that
        // ValueKey will start in the Union-Find array.
        let mut offsets = HashMap::from([(start, 0)]);
        let mut nodes = vec![start];
        let mut gates = HashSet::new();
        let mut n_bits = graph[start].bitsize.map_or(0, usize::from);
        let mut i = 0;
        while let Some(&node) = nodes.get(i) {
            i += 1;
            for p in &graph[node].links {
                if !graph.is_passive(*p) || !gates.insert(p.gate) { continue; }

                for (&link, props) in std::iter::zip(&graph[p.gate].links, &graph[p.gate].port_props) {
                    if props.ty != PortType::Passive { continue; }
                    // Value nodes with mismatched bitsizes aren't part of a net.
                    let Some((m, bitsize)) = link.and_then(|m| Some((m, graph[m].bitsize?))) else { continue };
                    if let hash_map::Entry::Vacant(e) = offsets.entry(m) {
                        e.insert(n_bits);
                        n_bits += usize::from(bitsize);
                        nodes.push(m);
                    }
                }
            }
        }
        resolved.extend(nodes.iter().copied());

        // Group the bits which passive components connect into nets
        let mut nets = UnionFind::new(n_bits);
        for &gate in &gates {
            let bit_index = |PortBit { index, bit }: PortBit| {
                graph[gate].links[index]
                    .and_then(|n| offsets.get(&n))
                    .map(|&offset| offset + usize::from(bit))
            };
            for (a, b) in graph[gate].func.passive_links() {
                if let (Some(a), Some(b)) = (bit_index(a), bit_index(b)) {
                    nets.union(a, b);
                }
            }
        }
        let roots: Vec<_> = (0..n_bits).map(|b| nets.find(b)).collect();

        // Resolve each net's value (forcing then normal driving)
        // RecalcState has a kind of special case here
        let is_forced = |node| matches!(batch.get(node), Some(PropagationState { recalculate: RecalcState::Pass }));
        // root --> value
        let mut net_states = vec![NetState::default(); n_bits];
        for &node in &nodes {
            let value = self.get_node_value(node);
            if is_forced(node) && Some(value.len()) == graph[node].bitsize {
                for (b, st) in std::iter::zip(offsets[&node].., value) {
                    net_states[roots[b]].force(st);
                }
            }
        }
        for &node in &nodes {
            let drivers = graph[node].links.iter()
                .filter(|p| graph[p.gate].port_props[p.index].ty.accepts_output())
                .map(|&p| self.get_port_value(p));
            for value in drivers {
                for (b, st) in std::iter::zip(offsets[&node].., value) {
                    net_states[roots[b]].drive(st);
                }
            }
        }

        // Update each value node from its nets
        let mut updated = vec![];
        for &node in &nodes {
            self[node].clear_issues();
            let recalculate = batch.get(node).map(|s| s.recalculate);
            if matches!(recalculate, Some(RecalcState::Pass)) {
                updated.push(node);
                continue;
            }

            let bits = offsets[&node]..offsets[&node] + graph[node].bitsize.map_or(0, usize::from);
            let result: BitArray = bits.clone()
                .map(|b| net_states[roots[b]].value())
                .collect();
            if bits.into_iter().any(|b| net_states[roots[b]].is_short()) {
                self[node].add_issue(ValueIssue::ShortCircuit);
            }

            let changed = self.get_node_value(node) != result;
            self[node].value = result;
            if changed || matches!(recalculate, Some(RecalcState::RecalcPass)) {
                updated.push(node);
            }
        }
        updated
    }
}

/// The resolved state of a single net (a set of bits connected by passive components).
#[derive(Clone, Copy, Default)]
enum NetState {
    /// No driver has driven a non-Z value to this net.
    #[default]
    Undriven,
    /// Exactly one driver has driven a non-Z value to this net.
    Driven(BitState),
    /// Multiple drivers have driven non-Z values to this net (holding the joined value).
    Short(BitState),
    /// This net's value was forced (see [`RecalcState::Pass`]) and ignores its drivers.
    Forced(BitState)
}
impl NetState {
    /// Joins two bit states (equivalent to [`BitArray::join`] for a single bit).
    fn join(a: BitState, b: BitState) -> BitState {
        match (a, b) {
            (BitState::Imped, st) | (st, BitState::Imped) => st,
            (a, b) if a == b => a,
            _ => BitState::Unk
        }
    }

    /// Gets the value of this net.
    fn value(self) -> BitState {
        match self {
            NetState::Undriven => BitState::Imped,
            NetState::Driven(st) | NetState::Short(st) | NetState::Forced(st) => st
        }
    }
    /// Checks whether this net is short circuited.
    fn is_short(self) -> bool {
        matches!(self, NetState::Short(_))
    }

    /// Joins a forced value into this net.
    fn force(&mut self, st: BitState) {
        let value = match *self {
            NetState::Forced(cur) => Self::join(cur, st),
            _ => st
        };
        *self = NetState::Forced(value);
    }
    /// Joins a driver's value into this net (if this net's value isn't forced).
    fn drive(&mut self, st: BitState) {
        if st == BitState::Imped { return; }
        *self = match *self {
            NetState::Undriven => NetState::Driven(st),
            NetState::Driven(cur) | NetState::Short(cur) => NetState::Short(Self::join(cur, st)),
            forced @ NetState::Forced(_) => forced
        };
    }
}

/// Disjoint set over `0..n`.
struct UnionFind {
    parent: Vec<usize>
}
impl UnionFind {
    fn new(n: usize) -> Self {
        Self { parent: (0..n).collect() }
    }

    fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]];
            x = self.parent[x];
        }
        x
    }

    fn union(&mut self, a: usize, b: usize) {
        let (ra, rb) = (self.find(a), self.find(b));
        self.parent[ra] = rb;
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
impl std::fmt::Debug for CircuitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use crate::engine::debug::DebugMap;

        f.debug_struct("CircuitState")
            .field("values", &DebugMap(&self.values))
            .field("functions", &DebugMap(&self.functions))
            .field("transient", &self.transient)
            .finish()
    }
}

/// The state of a [`ValueNode`].
/// 
/// [`ValueNode`]: super::graph::ValueNode
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
/// [`FunctionNode`]: super::graph::FunctionNode
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
    
    /// Gets the number of ports this function has
    pub fn get_num_ports(&self) -> usize {
        self.ports.len()
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
pub(crate) enum RecalcState {
    /// Don't recalculate this value and only propagate values over.
    Pass,
    /// Recalculate this value and only propagate if the value changes.
    Recalc,
    /// Recalculate this value and always propagate the value over.
    RecalcPass
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct PropagationState {
    /// Determines how values should be updated and propagated.
    pub(crate) recalculate: RecalcState,
}

/// Temporary propagation state.
#[derive(Default)]
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

    fn insert(&mut self, k: ValueKey, recalculate: RecalcState) {
        self.values.insert(k, PropagationState { recalculate });
    }
    pub fn mark_update(&mut self, k: ValueKey, recalculate: bool) {
        self.insert(k, match recalculate {
            true => RecalcState::RecalcPass,
            false => RecalcState::Pass,
        });
    }
}
impl std::fmt::Debug for TransientState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use crate::engine::debug::DebugMap;

        f.debug_struct("TransientState")
            .field("values", &DebugMap(&self.values))
            .field("functions", &self.functions)
            .finish()
    }
}