//! The middle end, which keeps track of component types, properties, and positions.
//! 
//! The main structs are:
//! - [`MiddleRepr`]: The main middle-end circuit representation.
//! - [`MiddleCircuit`]: A mutable view of one of the middle-end circuits.
//! 

use serde::de::value;
use slotmap::{SecondaryMap, SlotMap};
use thiserror::Error;

use crate::bitarray::{BitArray};
use crate::engine::graph::ValueNode;
use crate::engine::state::ValueIssue::{MismatchedBitsizes, OscillationDetected, ShortCircuit};
use crate::engine::state::{FunctionState, ValueState};
use crate::engine::{CircuitForest, CircuitKey, CircuitState, FunctionKey, FunctionPort, ValueKey};
use crate::middle_end::func::{ComponentBounds, Orientation, PhysicalComponent, PhysicalComponentEnum, PhysicalInitContext};
use crate::middle_end::string_interner::StringInterner;
use crate::middle_end::wire::{MeshKey, Wire, WireSet};

mod key;
mod string_interner;
#[cfg(feature="serde")]
pub mod serialize;
pub mod wire;
pub mod func;

pub use string_interner::TunnelSymbol;
pub use key::{ComponentKey, UIKey};

type Axis = u32;
type Coord = (Axis, Axis);

type AxisDelta = i32;
type CoordDelta = (AxisDelta, AxisDelta);

/// A group of middle circuits.
#[derive(Debug, Default)]
#[cfg_attr(feature="serde", derive(serde::Deserialize), serde(try_from = "serialize::CircuitFile"))]
pub struct MiddleRepr {
    engine: CircuitForest,
    physical: SecondaryMap<CircuitKey, CircuitArea>
}

/// A circuit's middle-end components and wires,
///   including their locations and properties.
#[derive(Debug, Default)]
struct CircuitArea {
    name: String,
    components: SecondaryMap<FunctionKey, ComponentProps>,
    ui_components: SlotMap<UIKey, ComponentProps>,
    wires: WireSet,
    tunnel_interner: StringInterner
}

/// Properties of a middle-end component.
#[derive(Debug)]
pub struct ComponentProps {
    pub label: String,
    pub label_location: Orientation,

    // Position
    pub origin: Coord,
    pub bounds: [Coord; 2],
    pub ports: Vec<Coord>,

    // Component-specific props
    pub inner: PhysicalComponentEnum
}

#[derive(Debug, Error)]
/// Errors which can occur when editing a middle-end circuit.
pub enum ReprEditErr {
    /// Component is out of bounds (so it cannot be added).
    #[error("component is out of bounds")]
    ComponentOutOfBounds,
    
    /// Component being specified doesn't exist (so it cannot be removed).
    #[error("component does not exist")]
    ComponentDoesNotExist,

    /// Adding a wire fails.
    #[error("cannot add wire")]
    CannotAddWire,
    /// Removing a wire fails.
    #[error("cannot remove wire")]
    CannotRemoveWire,
}

/// A mutable view of a middle-end circuit,
/// which includes its engine component ([`crate::engine::Circuit`])
/// and its physical properties.
#[derive(Debug)]
pub struct MiddleCircuit<'a> {
    repr: &'a mut MiddleRepr,
    key: CircuitKey
}
impl MiddleRepr {
    /// Creates a new middle representation.
    pub fn new() -> Self {
        Self::default()
    }
    /// Creates a new subcircuit.
    pub fn add_circuit(&mut self, name: &str) -> CircuitKey {
        let ck = self.engine.add_circuit();

        let area = CircuitArea {
            name: name.to_string(),
            ..Default::default()
        };
        self.physical.insert(ck, area);

        ck
    }

    /// Creates a mutable view for a given subcircuit.
    pub fn circuit(&mut self, key: CircuitKey) -> MiddleCircuit<'_> {
        MiddleCircuit { repr: self, key }
    } 
    //checks to see if a circuit with the given key exists in the middle end
    pub fn has_circuit(&self, key: CircuitKey) -> bool {
        self.physical.contains_key(key)
    }   
}

/// Basic macro to pretend Circuit has the "graph" and "state" fields.
/// 
/// This cannot be done with a function
/// because this is returning a place rather than a value.
macro_rules! circ {
    ($self:ident.engine)   => { $self.repr.engine.circuit($self.key) };
    ($self:ident.graph)    => { $self.repr.engine.graph($self.key) };
    ($self:ident.state)    => { $self.repr.engine.top_level_state($self.key) };
    ($self:ident.physical) => { $self.repr.physical[$self.key] };
}
impl MiddleCircuit<'_> {
    /// Adds a component to the circuit.
    /// 
    /// This takes the component, label, and location for the component.
    /// This returns [`ReprEditErr::ComponentOutOfBounds`] if it fails, which can occur if the component would be out of bounds. Otherwise, return the component key associated with added component.
    pub fn add_component<C: Into<PhysicalComponentEnum>>(&mut self, physical: C, label: &str, label_location: Orientation, pos: Coord) -> Result<ComponentKey, ReprEditErr> {
        let ctx = PhysicalInitContext { circuit: self, label };
        let physical = physical.into();
        let ComponentBounds { bounds, ports } = physical.init_bounds(ctx)
            .into_absolute(pos)
            .ok_or(ReprEditErr::ComponentOutOfBounds)?;
        let props = ComponentProps {
            label: label.to_string(),
            label_location,
            origin: pos,
            bounds,
            ports,
            inner: physical,
        };

        if let Some(component) = physical.init_engine() {
            // ~~~ Engine component ~~~
            let gate = circ!(self.engine).add_function_node(component);
            
            // Add port to wire set:
            for (index, &c) in props.ports.iter().enumerate() {
                let value = circ!(self.physical).wires.add_port(c, gate.into(), index, || circ!(self.engine).add_value_node())
                    .expect("Expected port addition to be successful");
                
                circ!(self.engine).connect_one(value, FunctionPort { gate, index });
            }

            circ!(self.physical).components.insert(gate, props);
            
            Ok(ComponentKey::Function(gate))
        } else {
            // ~~~ UI component ~~~

            // Add tunnel to wire set:
            if !props.label.is_empty() && matches!(props.inner, PhysicalComponentEnum::Tunnel(_)) {
                let &[coord] = props.ports.as_slice() else { unreachable!("Tunnel should have 1 port") };
                let sym = circ!(self.physical).tunnel_interner.add_ref(&props.label);
                circ!(self.physical).wires.add_tunnel(coord, sym, || circ!(self.engine).add_value_node());
            }

            let ui_key = circ!(self.physical).ui_components.insert(props);
            Ok(ComponentKey::UI(ui_key))
        }
    }


    /// Removes a component from the circuit.
    /// 
    /// This returns [`ReprEditErr::ComponentDoesNotExist`] if the component does not exist.
    pub fn remove_component(&mut self, key: ComponentKey) -> Result<(), ReprEditErr> {
        let props = match key {
            ComponentKey::Function(gate) => circ!(self.physical).components.remove(gate),
            ComponentKey::UI(key) => circ!(self.physical).ui_components.remove(key),
        }.ok_or(ReprEditErr::ComponentDoesNotExist)?;

        // Remove from engine (if applicable):
        if let ComponentKey::Function(gate) = key {
            let result = circ!(self.engine).remove_function_node(gate);
            debug_assert!(result, "Engine removal should succeed");
        }
        
        // Handle tunnels specially:
        if matches!(props.inner, PhysicalComponentEnum::Tunnel(_)) {
            let sym = circ!(self.physical).tunnel_interner.del_ref(&props.label)
                .expect("Tunnel should have an assigned symbol");
            circ!(self.physical).wires.remove_tunnel(props.origin, sym)
                .expect("Tunnel removal should succeed");
        } else {
            // Remove all ports from wire set:
            for index in 0..props.ports.len() {
                let result = circ!(self.physical).wires.remove_port(key, index)
                    .expect("Component removal should succeed");
                self.handle_remove(result);
            }
        }

        Ok(())
    }

    /// Adds a wire to the circuit and updates the circuit to properly accommodate the wire.
    /// 
    /// This function handles multiple cases:
    /// - If the new wire endpoint connects to the middle of a wire, the wire creates a junction on the intersecting wire.
    /// - If the new wire overlaps multiple wires, then only wires for the gaps will be created.
    /// 
    /// This raises an error if no wire is added.
    pub fn add_wire(&mut self, w: Wire) -> Result<(), ReprEditErr> {
        let result = circ!(self.physical).wires.add_wire(w, || circ!(self.engine).add_value_node())
            .ok_or(ReprEditErr::CannotAddWire)?;
        match result {
            wire::AddWireResult::NoJoin(_) => {},
            wire::AddWireResult::Join(c, keys) => if let &[k1, _, ..] = keys.as_slice() {
                circ!(self.engine).join(&keys);
                circ!(self.physical).wires.flood_fill(c, k1);
            },
        }

        Ok(())
    }

    /// Removes a wire to the circuit and updates the circuit
    /// to properly accommodate the removed wire.
    /// 
    /// This function removes any wires that overlap the wire range defined by the argument.
    pub fn remove_wire(&mut self, w: Wire) -> Result<(), ReprEditErr> {
        let result = circ!(self.physical).wires.remove_wire(w)
            .ok_or(ReprEditErr::CannotRemoveWire)?;

        self.handle_remove(result);

        Ok(())
    }

    /// Updates engine to corresponding `RemoveWireResult`.
    fn handle_remove(&mut self, result: wire::RemoveWireResult) {
        let wire::RemoveWireResult { deleted_keys, split_groups } = result;

        for k in deleted_keys {
            circ!(self.engine).remove_value_node(k);
        }
        for (k, groups) in split_groups {
            for group in &groups[1..] {
                let coord = group.iter()
                    .find_map(|&k| match k {
                        wire::MeshKey::WireJoint(c) => Some(c),
                        _ => None
                    })
                    .unwrap_or_else(|| unreachable!("Expected coordinate in split group"));
                
                // Get all ports associated with coordinates:
                let ports: Vec<_> = group.iter()
                    .filter_map(|&k| match k {
                        wire::MeshKey::Port(ComponentKey::Function(gate), index) => Some(FunctionPort { gate, index }),
                        _ => None
                    })
                    .collect();

                // Split and update physical:
                let flood_key = circ!(self.engine).split(k, &ports);
                circ!(self.physical).wires.flood_fill(coord, flood_key);
            }
        }
    }

    /// Updates the engine.
    pub fn propagate(&mut self) {
        circ!(self.engine).propagate();
    }

    /// Get the states of all components in the circuit.
    pub fn get_component_states<'a>(&'a self) -> Vec<(FunctionKey, &'a FunctionState)> {
        circ!(self.state)
            .functions
            .iter()
            .collect() 
    } 

    pub fn get_wire_states(&self)->Vec<(Wire, BitArray, Vec<String>)>{
       return circ!(self.physical).wires.wires().map(|wire| {
        let valueKey = circ!(self.physical).wires.find_key(MeshKey::from(wire.endpoints()[0])).unwrap();
        return (wire, circ!(self.state).get_node_value(valueKey), circ!(self.state).get_issues(valueKey).iter().map(|issue| {
            match issue{
                ShortCircuit =>"ShortCircuit".to_string(),
                OscillationDetected => "OscillationDetected".to_string(),
                MismatchedBitsizes => "MismatchedBitsize".to_string()
            }
        }).collect())
    }).collect();
    }
     pub fn get_wire_set(&self)->&WireSet{
        &circ!(self.physical).wires
    }
    pub fn get_circuit_state(&self)->&CircuitState{
        &circ!(self.state)
    }
      /// get the component properties for a given component key, returns an error if the component does not exist
    pub fn get_component(&self, key: ComponentKey) -> Result<&ComponentProps, ReprEditErr> {
        match key {
                ComponentKey::Function(gate) => circ!(self.physical).components.get(gate).ok_or(ReprEditErr::ComponentDoesNotExist),
            ComponentKey::UI(ui_key) => circ!(self.physical).ui_components.get(ui_key).ok_or(ReprEditErr::ComponentDoesNotExist),
        }
    }
    
    /// Checks to see if circuit has a component with the given key
    pub fn has_component(&self, key: ComponentKey) -> bool {
        match key {
            ComponentKey::Function(gate) => circ!(self.physical).components.contains_key(gate),
            ComponentKey::UI(ui_key) => circ!(self.physical).ui_components.contains_key(ui_key),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::func::BitSize;
    use crate::middle_end::func::Pin;

    use super::*;

    #[test]
    fn middle_repr_connect_wire() {
        let mut repr = MiddleRepr::new();
        let circuit_key = repr.add_circuit("Debug");
        let bitsize = BitSize::new(1).unwrap();
        let mut circuit = repr.circuit(circuit_key);

        let [p, q] = [(10, 10), (20, 10)];

        let left = circuit
            .add_component(Pin::new(bitsize, true, Orientation::East), "", Orientation::East, p)
            .unwrap();

        let right = circuit
            .add_component(Pin::new(bitsize, false, Orientation::East), "", Orientation::East, q)
            .unwrap();

        let w = Wire::from_endpoints(p, q).unwrap();
        circuit.add_wire(w).unwrap();

        let [lk, rk] = [left, right].map(|key| {
            let ComponentKey::Function(gate) = key else {
                panic!("expected function component");
            };
    
            circuit.get_wire_set()
                .find_key((ComponentKey::Function(gate), 0))
                .unwrap()
        });

        assert_eq!(lk, rk);
    }
    #[test]
    fn middle_repr_connect_wire_not_endpoint() {
        let mut repr = MiddleRepr::new();
        let circuit_key = repr.add_circuit("Debug");
        let bitsize = BitSize::new(1).unwrap();
        let mut circuit = repr.circuit(circuit_key);

        let [p, m1, m2, q] = [(10, 10), (15, 10), (25, 10), (30, 10)];
        let left = circuit
            .add_component(Pin::new(bitsize, true, Orientation::East), "", Orientation::East, p)
            .unwrap();

        let right = circuit
            .add_component(Pin::new(bitsize, false, Orientation::East), "", Orientation::East, q)
            .unwrap();

        circuit.add_wire(Wire::from_endpoints(p, m1).unwrap()).unwrap();
        circuit.add_wire(Wire::from_endpoints(m2, q).unwrap()).unwrap();
        circuit.add_wire(Wire::from_endpoints(m1, m2).unwrap()).unwrap();

        let [lk, rk] = [left, right].map(|key| {
            let ComponentKey::Function(gate) = key else {
                panic!("expected function component");
            };
    
            circuit.get_wire_set()
                .find_key((ComponentKey::Function(gate), 0))
                .unwrap()
        });

        assert_eq!(lk, rk);
    }
}