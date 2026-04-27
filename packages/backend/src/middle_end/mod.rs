//! The middle end, which keeps track of component types, properties, and positions.
//! 
//! The main structs are:
//! - [`MiddleRepr`]: The main middle-end circuit representation.
//! - [`MiddleCircuit`]: A mutable view of one of the middle-end circuits.
//! 


use slotmap::{SecondaryMap, SlotMap};

use crate::engine::state::{self, FunctionState};
use crate::engine::{CircuitForest, CircuitKey, FunctionKey, FunctionPort};
use crate::middle_end::func::{ComponentBounds, Handedness, Orientation, PhysicalComponent, PhysicalComponentEnum, PhysicalInitContext};
use crate::middle_end::string_interner::StringInterner;
use crate::middle_end::wire::{Wire, WireSet};

mod key;
mod string_interner;
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
pub struct MiddleRepr {
    engine: CircuitForest,
    physical: SecondaryMap<CircuitKey, CircuitArea>
}

/// A circuit's middle-end components and wires,
///   including their locations and properties.
#[derive(Debug, Default)]
struct CircuitArea {
    components: SecondaryMap<FunctionKey, ComponentProps>,
    ui_components: SlotMap<UIKey, ComponentProps>,
    wires: WireSet,
    tunnel_interner: StringInterner
}

/// Properties of a middle-end component.
#[derive(Debug)]
pub struct ComponentProps {
    label: String,

    // Position
    origin: Coord,
    bounds: [Coord; 2],
    ports: Vec<Coord>,
    orientation: Orientation,
    handedness: Handedness,

    // Extra props
    extra: PhysicalComponentEnum
}
impl ComponentProps {
    /// Getter for label
    pub fn label(&self) -> &str {
        &self.label
    }
    /// Getter for origin
    pub fn origin(&self) -> Coord {
        self.origin
    }
    /// Getter for bounds   
    pub fn bounds(&self) -> [Coord; 2] {
        self.bounds
    }
    /// Getter for ports
    pub fn ports(&self) -> &[Coord] {
        &self.ports
    }
    /// Getter for orientation
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }
        /// Getter for handedness
    pub fn handedness(&self) -> Handedness {
        self.handedness
    }
        /// Getter for extra properties
    pub fn extra(&self) -> &PhysicalComponentEnum {
        &self.extra
    }
}

/// Errors which can occur when editing a middle-end circuit.
pub enum ReprEditErr {
    /// Adding a component fails.
    CannotAddComponent,
    /// Removing a component fails.
    CannotRemoveComponent,
    /// Adding a wire fails.
    CannotAddWire,
    /// Removing a wire fails.
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
    /// Creates a mutable view for a given subcircuit.
    pub fn circuit(&mut self, key: CircuitKey) -> MiddleCircuit<'_> {
        MiddleCircuit { repr: self, key }
    }  
    /// Create a new circuit and return its key.
    pub fn add_circuit(&mut self) -> CircuitKey {
        let key = self.engine.add_circuit();
        self.physical.insert(key, CircuitArea::default());
        key
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
    ($self:ident.state)    => { $self.repr.engine.state($self.key) };
    ($self:ident.physical) => { $self.repr.physical[$self.key] };
}
impl MiddleCircuit<'_> {
    /// Adds a component to the circuit.
    /// 
    /// This takes the component, label, and location for the component.
    /// This returns [`ReprEditErr::CannotAddComponent`] if it fails, which can occur if the component would be out of bounds. Otherwise, return the component key associated with added component.
    pub fn add_component<C: Into<PhysicalComponentEnum>>(&mut self, physical: C, label: &str, pos: Coord) -> Result<ComponentKey, ReprEditErr> {
        let ctx = PhysicalInitContext { circuit: self, label };
        let physical = physical.into();
        let ComponentBounds { bounds, ports } = physical.bounds(ctx).into_absolute(pos)
            .ok_or(ReprEditErr::CannotAddComponent)?;
        let props = ComponentProps {
            label: label.to_string(),
            origin: pos,
            bounds,
            ports,
            orientation: Default::default(),
            handedness: Default::default(),
            extra: physical,
        };

        if let Some(component) = physical.engine_component() {
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
            if !props.label.is_empty() && matches!(props.extra, PhysicalComponentEnum::Tunnel(_)) {
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
    /// This returns [`ReprEditErr::CannotRemoveComponent`] if the component does not exist.
    pub fn remove_component(&mut self, key: ComponentKey) -> Result<(), ReprEditErr> {
        let props = match key {
            ComponentKey::Function(gate) => circ!(self.physical).components.remove(gate),
            ComponentKey::UI(key) => circ!(self.physical).ui_components.remove(key),
        }.ok_or(ReprEditErr::CannotRemoveComponent)?;

        // Remove from engine (if applicable):
        if let ComponentKey::Function(gate) = key {
            let result = circ!(self.engine).remove_function_node(gate);
            debug_assert!(result, "Engine removal should succeed");
        }
        
        // Handle tunnels specially:
        if matches!(props.extra, PhysicalComponentEnum::Tunnel(_)) {
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
            wire::AddWireResult::Join(c, k1, keys) => {
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
    /// Sets the orientation of a component, and updates the circuit to accommodate the new orientation.
    pub fn set_component_orientation(&mut self, key: ComponentKey, orientation: Orientation) -> Result<(), ReprEditErr> {
        //Extract component properties
        let props = match key {
            ComponentKey::Function(gate) => circ!(self.physical).components.get_mut(gate)
                .ok_or(ReprEditErr::CannotRemoveComponent)?,
            ComponentKey::UI(ui_key) => circ!(self.physical).ui_components.get_mut(ui_key)
                .ok_or(ReprEditErr::CannotRemoveComponent)?,
        };
        //extract properties that we will need to update:
        let (label, origin, old_ports, physical, handedness) = (
            props.label.clone(),
            props.origin,
            props.ports.clone(),
            props.extra,
            props.handedness,
        );

        // Get new bounds and ports: the extra property of props stores the physical component unaffected by orientation, so we can reuse it to get the new bounds and ports for the component with the new orientation.
        let ComponentBounds { bounds, ports } = physical
            .bounds(PhysicalInitContext { circuit: self, label: &label })
            .orient(orientation, handedness)
            .into_absolute(origin)
            .ok_or(ReprEditErr::CannotAddComponent)?;

        if !matches!(physical, PhysicalComponentEnum::Tunnel(_)) {
            // Tunnel port is unaffected by orientation changes bc there is only one port, however for both ui and engine componets there are multiple ports whose positions are affected
            for index in 0..old_ports.len() {
                    let result = circ!(self.physical).wires.remove_port(key, index)
                        .expect("Component removal should succeed");
                    self.handle_remove(result);
                }
        } 
            
        
        // Add new ports to wire set:
        if let ComponentKey::Function(gate) = key {
            for (index, &coord) in ports.iter().enumerate() {
                let value = circ!(self.physical).wires.add_port(coord, key, index, || circ!(self.engine).add_value_node())
                    .expect("Expected port addition to be successful");
                circ!(self.engine).connect_one(value, FunctionPort { gate, index });
            }
        } 
            // Update component properties:
        match key {
            ComponentKey::Function(gate) => {
                let props = circ!(self.physical).components.get_mut(gate)
                    .ok_or(ReprEditErr::CannotRemoveComponent)?;
                props.bounds = bounds;
                props.ports = ports;
                props.orientation = orientation;
            }
            ComponentKey::UI(ui_key) => {
                let props = circ!(self.physical).ui_components.get_mut(ui_key)
                    .ok_or(ReprEditErr::CannotRemoveComponent)?;
                props.bounds = bounds;
                props.ports = ports;
                props.orientation = orientation;
            }
        }

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

    /// Gets the states of all components in the circuit
    pub fn get_component_states<'a>(&'a self) -> Vec<(FunctionKey, &'a FunctionState)> {
        circ!(self.state)
            .functions
            .iter()
            .collect() 
    }
    /// get the component properties for a given component key, returns an error if the component does not exist
    pub fn get_component(&self, key: ComponentKey) -> Result<&ComponentProps, ReprEditErr> {
        match key {
            ComponentKey::Function(gate) => circ!(self.physical).components.get(gate)
                .ok_or(ReprEditErr::CannotRemoveComponent),
            ComponentKey::UI(ui_key) => circ!(self.physical).ui_components.get(ui_key)
                .ok_or(ReprEditErr::CannotRemoveComponent),
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
