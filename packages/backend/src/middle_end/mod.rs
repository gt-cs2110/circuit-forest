//! The middle end, which keeps track of component types, properties, and positions.
//! 
//! The main structs are:
//! - [`MiddleRepr`]: The main middle-end circuit representation.
//! - [`MiddleCircuit`]: A mutable view of one of the middle-end circuits.
//! 

use slotmap::{SecondaryMap};
use thiserror::Error;

use crate::engine::{CircuitForest, CircuitKey, CircuitState, FunctionPort};
use crate::middle_end::comp_key::ComponentMap;
use crate::middle_end::func::{AbsoluteComponentBounds, ComponentBounds, Orientation, PhysicalComponent, PhysicalComponentEnum, PhysicalInitContext};
use crate::middle_end::string_interner::StringInterner;
use crate::middle_end::wire::{Wire, WireSet};

mod comp_key;
mod string_interner;
#[cfg(feature="serde")]
pub mod serialize;
pub mod wire;
pub mod func;

pub use string_interner::TunnelSymbol;
pub use comp_key::{ComponentKey, UIKey};

type Axis = u32;
type Coord = (Axis, Axis);

type AxisDelta = i32;
type CoordDelta = (AxisDelta, AxisDelta);

/// A group of middle circuits.
#[derive(Default)]
#[cfg_attr(feature="serde", derive(serde::Deserialize), serde(try_from = "serialize::CircuitFile"))]
pub struct MiddleRepr {
    engine: CircuitForest,
    physical: SecondaryMap<CircuitKey, CircuitArea>
}
impl std::fmt::Debug for MiddleRepr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use crate::engine::debug::DebugMap;

        f.debug_struct("MiddleRepr")
            .field("engine", &self.engine)
            .field("physical", &DebugMap(&self.physical))
            .finish()
    }
}
/// A circuit's middle-end components and wires,
///   including their locations and properties.
#[derive(Debug, Default)]
struct CircuitArea {
    name: String,
    components: ComponentMap<ComponentProps>,
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
    
    /// Adding a tunnel at a spot where the same tunnel already exists.
    #[error("cannot place two identical tunnels with the same port")]
    RedundantTunnel,

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

    /// Creates a mutable view for a given subcircuit,
    /// panicking if the circuit does not exist.
    pub fn circuit(&mut self, key: CircuitKey) -> MiddleCircuit<'_> {
        self.try_circuit(key)
            .unwrap_or_else(|| panic!("circuit of key {key:?} does not exist"))
    } 

    /// Tries to create a mutable view for a given subcircuit,
    /// returning None if the circuit does not exist.
    pub fn try_circuit(&mut self, key: CircuitKey) -> Option<MiddleCircuit<'_>> {
        match self.physical.contains_key(key) {
            true => Some(MiddleCircuit { repr: self, key }),
            false => None
        }
    }
}

/// Basic macro to pretend Circuit has the "graph" and "state" fields.
/// 
/// This cannot be done with a function
/// because this is returning a place rather than a value.
macro_rules! circ {
    ($self:ident.engine)   => { $self.repr.engine.circuit($self.key) };
    ($self:ident.graph)    => { $self.repr.engine.graphs[$self.key] };
    ($self:ident.state)    => { $self.repr.engine.states[$self.key] };
    ($self:ident.physical) => { $self.repr.physical[$self.key] };
}
impl MiddleCircuit<'_> {
    /// Adds a component to the circuit.
    /// 
    /// This takes the component, label, and location for the component.
    /// This returns [`ReprEditErr::ComponentOutOfBounds`] if it fails, which can occur if the component would be out of bounds. Otherwise, return the component key associated with added component.
    pub fn add_component<C: Into<PhysicalComponentEnum>>(&mut self, physical: C, label: &str, label_location: Orientation, pos: Coord) -> Result<ComponentKey, ReprEditErr> {
        let physical = physical.into();
        let ComponentBounds { bounds, ports } = self.validate_bounds(physical, label, pos)?;
        let props = ComponentProps {
            label: label.to_string(),
            label_location,
            origin: pos,
            bounds,
            ports,
            inner: physical,
        };

        let key: ComponentKey = if let Some(component) = physical.init_engine() {
            // ~~~ Engine component ~~~
            let gate = circ!(self.engine).add_function_node(component);
            circ!(self.physical).components.func.insert(gate, props);
            gate.into()
        } else {
            // ~~~ UI component ~~~
            let ui_key = circ!(self.physical).components.ui.insert(props);
            ui_key.into()
        };

        // Update the wire set to include all the component's ports.
        //    For tunnels, all tunnels are treated as one unified port.
        //    For every other type of component, each port is passed onto the wire set.
        let CircuitArea { name: _, components, wires, tunnel_interner } = &mut circ!(self.physical);
        let props = &components[key];
        if matches!(props.inner, PhysicalComponentEnum::Tunnel(_)) {
            if !props.label.is_empty() {
            // Add tunnel to wire set:
                let &[coord] = props.ports.as_slice() else { unreachable!("Tunnel should have 1 port") };
                let sym = tunnel_interner.add_ref(&props.label);
                let result = wires.add_tunnel(coord, sym, || circ!(self.engine).add_value_node())
                    .ok_or(ReprEditErr::RedundantTunnel)?;
                self.handle_add(result);
            }
        } else {
            // Add port to wire set:
            for (index, &c) in props.ports.iter().enumerate() {
                let value = wires.add_port(c, key, index, || circ!(self.engine).add_value_node())
                    .expect("Expected port addition to be successful");
                
                if let ComponentKey::Function(gate) = key {
                    circ!(self.engine).connect_one(value, FunctionPort { gate, index });
                }
            }
        }
        
        Ok(key)
    }


    /// Removes a component from the circuit.
    /// 
    /// This returns [`ReprEditErr::ComponentDoesNotExist`] if the component does not exist.
    pub fn remove_component(&mut self, key: ComponentKey) -> Result<(), ReprEditErr> {
        let props = circ!(self.physical).components.remove(key)
            .ok_or(ReprEditErr::ComponentDoesNotExist)?;

        // Remove from engine (if applicable):
        if let ComponentKey::Function(gate) = key {
            let result = circ!(self.engine).remove_function_node(gate);
            debug_assert!(result, "Engine removal should succeed");
        }
        
        // Handle tunnels specially:
        if matches!(props.inner, PhysicalComponentEnum::Tunnel(_)) {
            if !props.label.is_empty() {
                let sym = circ!(self.physical).tunnel_interner.del_ref(&props.label)
                    .expect("Tunnel should have an assigned symbol");
    
                let result = circ!(self.physical).wires.remove_tunnel(props.origin, sym)
                    .expect("Tunnel removal should succeed");
                self.handle_remove(result);
            }
            
        } else {
            // Remove all ports from wire set:
            for index in 0..props.ports.len() {
                let result = circ!(self.physical).wires.remove_port(key, index)
                    .expect("Port removal should succeed");
                self.handle_remove(result);
            }
        }

        Ok(())
    }

    /// Validates whether a certain placement configuration is in bounds.
    pub fn validate_bounds(&self, physical: PhysicalComponentEnum, label: &str, pos: Coord) -> Result<AbsoluteComponentBounds, ReprEditErr> {
        let ctx = PhysicalInitContext { circuit: self, label };
        physical.init_bounds(ctx)
            .into_absolute(pos)
            .ok_or(ReprEditErr::ComponentOutOfBounds)
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

        self.handle_add(result);

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

    /// Updates engine & middle end to corresponding AddWireResult.
    fn handle_add(&mut self, result: wire::AddWireResult) {
        match result {
            wire::AddWireResult::NoJoin(_) => {},
            wire::AddWireResult::Join(c, keys) => if let &[k1, _, ..] = keys.as_slice() {
                circ!(self.engine).join(&keys);
                circ!(self.physical).wires.flood_fill(c, k1);
            },
        }
    }
    /// Updates engine & middle end to corresponding `RemoveWireResult`.
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

    pub fn get_wire_set(&self) -> &WireSet {
        &circ!(self.physical).wires
    }
    pub fn get_circuit_state(&self) -> &CircuitState {
        &circ!(self.state)
    }
    
    /// Get the component properties for a given component key, returning an error if the component does not exist.
    pub fn get_component(&self, key: ComponentKey) -> Result<&ComponentProps, ReprEditErr> {
        circ!(self.physical).components.get(key).ok_or(ReprEditErr::ComponentDoesNotExist)
    }
    
    /// Checks to see if circuit has a component with the given key
    pub fn has_component(&self, key: ComponentKey) -> bool {
        circ!(self.physical).components.contains_key(key)
    }

    pub fn get_components(&self) -> impl Iterator<Item=(ComponentKey, &ComponentProps)> {
        circ!(self.physical).components.iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::middle_end::func::Pin;

    use super::*;

    #[test]
    fn middle_repr_connect_wire() {
        let mut repr = MiddleRepr::new();
        let circuit_key = repr.add_circuit("Debug");
        let mut circuit = repr.circuit(circuit_key);

        let [p, q] = [(10, 10), (20, 10)];

        let left = circuit
            .add_component(Pin::new(1, true, Orientation::East), "", Orientation::East, p)
            .unwrap();

        let right = circuit
            .add_component(Pin::new(1, false, Orientation::East), "", Orientation::East, q)
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
        let mut circuit = repr.circuit(circuit_key);

        let [p, m1, m2, q] = [(10, 10), (15, 10), (25, 10), (30, 10)];
        let left = circuit
            .add_component(Pin::new(1, true, Orientation::East), "", Orientation::East, p)
            .unwrap();

        let right = circuit
            .add_component(Pin::new(1, false, Orientation::East), "", Orientation::East, q)
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