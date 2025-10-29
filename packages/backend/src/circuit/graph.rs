//! Types for describing the structure of a circuit.
//! 
//! This module notably includes:
//! - [`CircuitGraph`]: The main structure for a circuit
//! - [`ValueNode`]: Nodes which represent wires
//! - [`FunctionNode`]: Nodes which represent components

use std::collections::HashSet;
use std::ops::{Index, IndexMut};

use slotmap::{SlotMap, new_key_type};

use crate::node::{Component, ComponentFn, PortProperties};


new_key_type! {
    /// Key type for maps to values.
    pub struct ValueKey;
    /// Key type for maps to functions.
    pub struct FunctionKey;
}

/// A struct which identifies a port (from its function node and port index).
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug)]
pub struct FunctionPort {
    /// The function node which is associated with this port.
    pub gate: FunctionKey,
    /// The port's index on the function node.
    pub index: usize
}

/// A node which represents a wire (which can hold some value).
/// 
/// This node may only connect to function node ports.
#[derive(Default, Debug)]
pub struct ValueNode {
    /// The current bitsize.
    /// 
    /// This is used to check for bitsize mismatches.
    /// This is None if there is nothing or mismatched bitsizes
    /// connecting to or from this node.
    pub(crate) bitsize: Option<u8>,

    /// Ports this node is connected to.
    pub(crate) links: HashSet<FunctionPort>
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
#[derive(Debug)]
pub struct FunctionNode {
    /// The actual function that is applied.
    pub(crate) func: ComponentFn,
    /// The properties of this function's ports.
    pub(crate) port_props: Vec<PortProperties>,
    // Value nodes each port is connected to (if connected to a port).
    pub(crate) links: Vec<Option<ValueKey>>,
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
#[derive(Default, Debug)]
pub struct CircuitGraph {
    /// All value nodes of the circuit.
    pub(crate) values: SlotMap<ValueKey, ValueNode>,
    /// All function nodes of the circuit.
    pub(crate) functions: SlotMap<FunctionKey, FunctionNode>,
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
