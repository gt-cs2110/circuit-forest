//! Circuit module used to create circuits/sub-circuits.

pub mod state;
pub mod graph;

use slotmap::{SecondaryMap, SlotMap, new_key_type};

use crate::bitarray::BitArray;
use crate::circuit::graph::{CircuitGraph, FunctionKey, FunctionNode, FunctionPort, ValueKey};
use crate::circuit::state::{CircuitState, PropagationState};
use crate::func::ComponentFn;

new_key_type! {
    /// Key type for maps to circuits.
    pub struct CircuitKey;
}

/// The map of circuit graphs.
pub type CircuitGraphMap = SlotMap<CircuitKey, CircuitGraph>;
/// A group of circuits.
/// 
/// This encompasses multiple circuits.
/// Circuits within this CircuitForest 
/// can use any circuit within the forest
/// as a subcircuit.
#[derive(Default, Debug)]
pub struct CircuitForest {
    graphs: SlotMap<CircuitKey, CircuitGraph>,
    states: SecondaryMap<CircuitKey, CircuitState>,
}
impl CircuitForest {
    /// Creates an empty [`CircuitForest`].
    pub fn new() -> Self {
        Default::default()
    }

    /// Adds a new circuit to the forest and returns the circuit key to it.
    pub fn add_circuit(&mut self) -> CircuitKey {
        let key = self.graphs.insert(Default::default());
        self.states.insert(key, Default::default());
        key
    }

    /// Adds a new circuit to the forest and returns a mutable reference to it.
    pub fn new_circuit(&mut self) -> Circuit<'_> {
        let key = self.add_circuit();
        self.circuit(key)
    }

    /// Gets a mutable reference to the circuit specified by the key.
    pub fn circuit(&mut self, key: CircuitKey) -> Circuit<'_> {
        Circuit { forest: self, key }
    }

    /// Gets the graph associated with this key.
    pub fn graph(&self, k: CircuitKey) -> &CircuitGraph {
        &self.graphs[k]
    }
    /// Gets the top level state associated with this key.
    pub fn top_level_state(&self, k: CircuitKey) -> &CircuitState {
        &self.states[k]
    }
}

/// Issues which can occur to a value node.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ValueIssue {
    /// Represents a collision of bit values (short circuit).
    ShortCircuit,

    /// Represents a connection with two different bitsizes.
    MismatchedBitsizes,

    /// Represents a connection whose value is unstable.
    OscillationDetected
}

/// A mutable view of a circuit, 
/// which includes its structure ([`CircuitGraph`]) and its state ([`CircuitState`]).
#[derive(Debug)]
pub struct Circuit<'a> {
    forest: &'a mut CircuitForest,
    key: CircuitKey
}

/// Basic macro to pretend Circuit has the "graph" and "state" fields.
/// 
/// This cannot be done with a function
/// because this is returning a place rather than a value.
macro_rules! circ {
    ($self:ident.$field:ident) => { $self.forest.$field[$self.key] }
}

impl Circuit<'_> {
    /// Adds an input function node and value node (a wire connecting from the input)
    /// using the passed value.
    pub fn add_input(&mut self, arr: BitArray) -> (FunctionKey, ValueKey) {
        let func = self.add_function_node(crate::func::Input::new(arr.len()));
        let value = self.add_value_node();
        
        let result = self.replace_port(FunctionPort { gate: func, index: 0 }, arr);
        debug_assert!(result.is_ok(), "Port was set to value with incorrect bitsize");

        self.connect_all(func, &[value]);

        (func, value)
    }
    
    /// Adds an input function node and value node (a wire connecting from the input)
    /// using the passed value.
    pub fn add_output(&mut self, bitsize: u8) -> (FunctionKey, ValueKey) {
        let func = self.add_function_node(crate::func::Output::new(bitsize));
        let value = self.add_value_node();

        self.connect_all(func, &[value]);

        (func, value)
    }
    
    /// Create a value node (essentially a wire) with the specified bitsize.
    pub fn add_value_node(&mut self) -> ValueKey {
        let key = circ!(self.graphs).add_value();
        circ!(self.states).init_value(key);
        key
    }

    /// Create a function node with the passed component function.
    pub fn add_function_node<F: Into<ComponentFn>>(&mut self, f: F) -> FunctionKey {
        let node = FunctionNode::new(f.into(), &self.forest.graphs);
        let key = circ!(self.graphs).add_function(node);
        circ!(self.states).init_func(key, &circ!(self.graphs).functions[key].func, &self.forest.graphs);
        key
    }

    /// Removes a value node and any state linking to it.
    pub fn remove_value_node(&mut self, value: ValueKey) -> bool {
        if let Some(vnode) = circ!(self.graphs).values.get(value) {
            // Update transient state:
            circ!(self.states).transient.functions.extend(vnode.links.iter().map(|l| l.gate));
            
            circ!(self.graphs).remove_value(value);
            circ!(self.states).remove_node_value(value);
            true
        } else {
            false
        }
    }

    /// Removes a value node and any state linking to it.
    pub fn remove_function_node(&mut self, gate: FunctionKey) -> bool {
        if let Some(fnode) = circ!(self.graphs).functions.get(gate) {
            // Update transient state:
            for &ml in &fnode.links {
                if let Some(l) = ml {
                    circ!(self.states).add_transient(l, true);
                }
            }
            
            circ!(self.graphs).remove_function(gate);
            circ!(self.states).remove_function_value(gate);
            true
        } else {
            false
        }
    }

    /// Connect a wire to a port in the Circuit's graph.
    pub fn connect_one(&mut self, wire: ValueKey, port: FunctionPort) {
        circ!(self.graphs).connect(wire, port);
        circ!(self.states).add_transient(wire, true);
        self.propagate();
    }

    /// Clear the function node of connections and connect all of the passed ports to it.
    pub fn connect_all(&mut self, gate: FunctionKey, ports: &[ValueKey]) {
        circ!(self.graphs).clear_edges(gate);
        ports.iter().copied()
            .enumerate()
            .for_each(|(index, wire)| {
                circ!(self.graphs).connect(wire, FunctionPort { gate, index });
                circ!(self.states).add_transient(wire, true);
            });
        self.propagate();
    }

    /// Propagates an update through the circuit
    /// (until the circuit stabilizes or an oscillation occurs).
    /// 
    /// The provided `input` argument indicates which value nodes were updated.
    pub fn run(&mut self, inputs: &[ValueKey]) {
        circ!(self.states).transient.values = inputs.iter().copied()
            .map(|k| (k, PropagationState { recalculate: false }))
            .collect();
        circ!(self.states).transient.functions.clear();

        self.propagate();
    }

    /// Pushes transient state, propagating any updates through
    /// (until the circuit stabilizes or an oscillation occurs).
    pub fn propagate(&mut self) {
        circ!(self.states).propagate(&self.forest.graphs, self.key);
    }

    /// Gets current circuit state.
    pub fn state(&self) -> &CircuitState {
        &circ!(self.states)
    }

    /// Updates a [`ValueNode`] with the specified value, raising `Err` if bitsizes do not match.
    /// 
    /// If [`Circuit::propagate`] is called after this, the update propagates to the rest of the graph.
    /// 
    /// [`ValueNode`]: crate::circuit::graph::ValueNode
    pub fn replace_value(&mut self, key: ValueKey, val: BitArray) -> Result<(), crate::bitarray::MismatchedBitsizes> {
        circ!(self.states)[key].replace_value(val)?;

        circ!(self.states).add_transient(key, false);
        Ok(())
    }
    /// Updates the port of a [`FunctionNode`] with the specified value, raising `Err` if bitsizes do not match.
    /// 
    /// If [`Circuit::propagate`] is called after this, the update propagates to the rest of the graph.
    pub fn replace_port(&mut self, port: FunctionPort, val: BitArray) -> Result<(), crate::bitarray::MismatchedBitsizes> {
        circ!(self.states)[port.gate].replace_port(port.index, val)?;

        if let Some(wire) = circ!(self.graphs)[port.gate].links[0] {
            circ!(self.states).add_transient(wire, true);
        }
        Ok(())
    }

    /// Joins a set of value nodes into one.
    pub fn join(&mut self, keys: &[ValueKey]) {
        if let Some((&main, to_merge)) = keys.split_first() {
            circ!(self.graphs).join(main, to_merge);
            // Remove all invalidated nodes
            for &k in to_merge {
                circ!(self.states).remove_node_value(k);
            }

            circ!(self.states).add_transient(main, true);
        }
    }
    /// Splits a node into two and returns the newly created node.
    pub fn split(&mut self, key: ValueKey, off_ports: &[FunctionPort]) -> ValueKey {
        let new_value = circ!(self.graphs).split_off(key, off_ports);

        circ!(self.states).init_value(new_value);
        circ!(self.states).add_transient(key, true);
        circ!(self.states).add_transient(new_value, true);

        new_value
    }

    /// Sets the input state for the input.
    #[allow(dead_code)]
    pub(crate) fn set_input(&mut self, key: FunctionKey, value: BitArray) -> Result<(), crate::bitarray::MismatchedBitsizes> {
        assert!(matches!(circ!(self.graphs).functions[key].func, ComponentFn::Input(_)), "Expected input function");
        
        self.replace_port(FunctionPort { gate: key, index: 0 }, value)?;
        // FIXME: Have alternative which doesn't propagate
        self.propagate();

        Ok(())
    }

    /// Sets the input state for the input.
    #[allow(dead_code)]
    pub(crate) fn get_output(&mut self, key: FunctionKey) -> BitArray {
        assert!(matches!(circ!(self.graphs).functions[key].func, ComponentFn::Output(_)), "Expected output function");
        circ!(self.states).get_port_value(FunctionPort { gate: key, index: 0 })
    }
}

#[cfg(test)]
mod tests {
    use crate::bitarr;
    use crate::circuit::CircuitForest;
    use crate::circuit::graph::FunctionPort;
    use crate::func::{And, Not, ComponentFn};

    #[test]
    fn test_replace() {
        let mut forest = CircuitForest::new();
        let mut circuit = forest.new_circuit();
        
        let value = bitarr![0; 8]; // Create a 8 bit BitArray 
        let key = circuit.add_value_node();

        // replace should succeed
        assert!(circuit.replace_value(key, value).is_ok());

        // intentionally wrong bitsize should fail
        let wrong_value = bitarr![0; 16]; // 16 bits instead of 8
        assert!(circuit.replace_value(key, wrong_value).is_err());
    }
    #[test]
    fn test_join() {
        let mut forest = CircuitForest::new();
        let mut circuit = forest.new_circuit();

        let value1 = circuit.add_value_node();
        let value2 = circuit.add_value_node();

        let and =  ComponentFn::And(And::new(8, 2));
        let not= ComponentFn::Not(Not::new(8));

        let func1 = circuit.add_function_node(and);
        let func2 = circuit.add_function_node(not);

        // Connect value1 to func1, value2 to func2
        circuit.connect_one(value1, FunctionPort { gate: func1, index: 0 });
        circuit.connect_one(value2, FunctionPort { gate: func2, index: 0 });

        // Join value2 into value1
        circuit.join(&[value1, value2]);

        assert_eq!(circuit.forest.graphs[circuit.key].values[value1].links.len(), 2);
        assert!(!circuit.state().values.contains_key(value2));
        assert!(circuit.state().values.contains_key(value1));

        // Verify transients
        assert!(circuit.state().transient.values.contains_key(value1));
    }
    #[test]
    fn test_split_off() {
        let mut forest = CircuitForest::new();
        let mut circuit = forest.new_circuit();

        let value1 = circuit.add_value_node();

        let and =  ComponentFn::And(And::new(8, 2));
        let not= ComponentFn::Not(Not::new(8));

        let func1 = circuit.add_function_node(and);
        let func2 = circuit.add_function_node(not);

        circuit.connect_one(value1, FunctionPort { gate: func1, index: 0 });
        circuit.connect_one(value1, FunctionPort { gate: func2, index: 0 });

        // Split off func2 from value1
        let value2 = circuit.split(value1, &[FunctionPort { gate: func2, index: 0 }]);

        // Verify graphs and state
        assert_eq!(circuit.forest.graphs[circuit.key].values[value1].links.len(), 1);
        assert_eq!(circuit.forest.graphs[circuit.key].values[value2].links.len(), 1);

        assert!(circuit.state().values.contains_key(value1));
        assert!(circuit.state().values.contains_key(value2));

        // Verify transients
        assert!(circuit.state().transient.values.contains_key(value1));
        assert!(circuit.state().transient.values.contains_key(value2));
    }
}
