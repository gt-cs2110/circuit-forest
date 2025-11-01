//! Circuit module used to create circuits/sub-circuits.

pub mod state;
pub mod graph;

use slotmap::{SecondaryMap, SlotMap, new_key_type};

use crate::bitarray::BitArray;
use crate::circuit::graph::{CircuitGraph, FunctionKey, FunctionNode, FunctionPort, ValueKey};
use crate::circuit::state::{CircuitState, TriggerState};
use crate::func::ComponentFn;

new_key_type! {
    /// Key type for maps to circuits.
    pub struct CircuitKey;
}
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

/// A circuit, which includes its structure ([`CircuitGraph`]) and its state ([`CircuitState`]).
#[derive(Debug)]
pub struct Circuit<'a> {
    forest: &'a mut CircuitForest,
    key: CircuitKey
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
        let key = self.forest.graphs[self.key].add_value();
        self.forest.states[self.key].init_value(key);
        key
    }

    /// Create a function node with the passed component function.
    pub fn add_function_node<F: Into<ComponentFn>>(&mut self, f: F) -> FunctionKey {
        let key = self.forest.graphs[self.key].add_function(f.into());
        self.forest.states[self.key].init_func(key, &self.forest.graphs[self.key].functions[key].func);
        key
    }

    /// Connect a wire to a port in the Circuit's graph.
    pub fn connect_one(&mut self, wire: ValueKey, port: FunctionPort) {
        self.forest.graphs[self.key].connect(wire, port);
        self.forest.states[self.key].transient.triggers.insert(wire, TriggerState { recalculate: true });
        self.propagate();
    }

    /// Clear the function node of connections and connect all of the passed ports to it.
    pub fn connect_all(&mut self, gate: FunctionKey, ports: &[ValueKey]) {
        self.forest.graphs[self.key].clear_edges(gate);
        ports.iter().copied()
            .enumerate()
            .for_each(|(index, wire)| {
                self.forest.graphs[self.key].connect(wire, FunctionPort { gate, index });
                self.forest.states[self.key].transient.triggers.insert(wire, TriggerState { recalculate: true });
            });
        self.propagate();
    }

    /// Propagates an update through the circuit
    /// (until the circuit stabilizes or an oscillation occurs).
    /// 
    /// The provided `input` argument indicates which value nodes were updated.
    pub fn run(&mut self, inputs: &[ValueKey]) {
        self.forest.states[self.key].transient.triggers = inputs.iter().copied()
            .map(|k| (k, TriggerState { recalculate: false }))
            .collect();
        self.forest.states[self.key].transient.frontier.clear();

        self.propagate();
    }

    /// Pushes transient state, propagating any updates through
    /// (until the circuit stabilizes or an oscillation occurs).
    pub fn propagate(&mut self) {
        self.forest.states[self.key].propagate(&self.forest.graphs[self.key]);
    }

    /// Gets current circuit state.
    pub fn state(&self) -> &CircuitState {
        &self.forest.states[self.key]
    }

    /// Updates a [`ValueNode`] with the specified value, raising `Err` if bitsizes do not match.
    /// 
    /// If [`Circuit::propagate`] is called after this, the update propagates to the rest of the graph.
    pub fn replace_value(&mut self, key: ValueKey, val: BitArray) -> Result<(), crate::bitarray::MismatchedBitsizes> {
        self.forest.states[self.key][key].replace_value(val)?;

        self.forest.states[self.key].transient.triggers.insert(key, TriggerState { recalculate: false });
        Ok(())
    }
    /// Updates the port of a [`FunctionNode`] with the specified value, raising `Err` if bitsizes do not match.
    /// 
    /// If [`Circuit::propagate`] is called after this, the update propagates to the rest of the graph.
    pub fn replace_port(&mut self, port: FunctionPort, val: BitArray) -> Result<(), crate::bitarray::MismatchedBitsizes> {
        self.forest.states[self.key][port.gate].replace_port(port.index, val)?;

        if let Some(wire) = self.forest.graphs[self.key][port.gate].links[0] {
            self.forest.states[self.key].transient.triggers.insert(wire, TriggerState { recalculate: true });
        }
        Ok(())
    }

    /// Sets the input state for the input.
    #[allow(dead_code)]
    pub(crate) fn set_input(&mut self, key: FunctionKey, value: BitArray) -> Result<(), crate::bitarray::MismatchedBitsizes> {
        assert!(matches!(self.forest.graphs[self.key].functions[key].func, ComponentFn::Input(_)), "Expected input function");
        
        self.replace_port(FunctionPort { gate: key, index: 0 }, value)?;
        // FIXME: Have alternative which doesn't propagate
        self.propagate();

        Ok(())
    }

    /// Sets the input state for the input.
    #[allow(dead_code)]
    pub(crate) fn get_output(&mut self, key: FunctionKey) -> BitArray {
        assert!(matches!(self.forest.graphs[self.key].functions[key].func, ComponentFn::Output(_)), "Expected output function");
        self.forest.states[self.key].get_port_value(FunctionPort { gate: key, index: 0 })
    }
}

#[cfg(test)]
mod tests {
    use crate::bitarr;
    use crate::circuit::CircuitForest;

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
}
