use slotmap::{SecondaryMap, SlotMap};

use crate::engine::{CircuitForest, CircuitKey, FunctionKey, FunctionPort};
use crate::middle_end::func::{ComponentBounds, PhysicalComponent, PhysicalComponentEnum, PhysicalInitContext};
use crate::middle_end::string_interner::StringInterner;
use crate::middle_end::wire::{Wire, WireSet};

mod key;
mod serialize;
mod string_interner;
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
pub struct CircuitArea {
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

    // Extra props
    extra: PhysicalComponentEnum
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
/// and its physical properties ([`CircuitArea`]).
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
}

/// Basic macro to pretend Circuit has the "graph" and "state" fields.
/// 
/// This cannot be done with a function
/// because this is returning a place rather than a value.
macro_rules! circ {
    ($self:ident.engine)   => { $self.repr.engine.circuit($self.key) };
    ($self:ident.graph)    => { $self.repr.engine.graphs[$self.key] };
    ($self:ident.state)    => { $self.repr.engine.state[$self.key] };
    ($self:ident.physical) => { $self.repr.physical[$self.key] };
}
impl MiddleCircuit<'_> {
    /// Adds a component to the circuit.
    /// 
    /// This takes the component, label, and location for the component.
    /// This returns [`ReprEditErr::CannotAddComponent`] if it fails, which can occur if the component would be out of bounds.
    pub fn add_component<C: Into<PhysicalComponentEnum>>(&mut self, physical: C, label: &str, pos: Coord) -> Result<(), ReprEditErr> {
        let ctx = PhysicalInitContext { circuit: self, label };
        let physical = physical.into();
        let ComponentBounds { bounds, ports } = physical.bounds(ctx).into_absolute(pos)
            .ok_or(ReprEditErr::CannotAddComponent)?;
        let props = ComponentProps {
            label: label.to_string(),
            origin: pos,
            bounds,
            ports,
            extra: physical,
        };

        if let Some(component) = physical.engine_component() {
            // ~~~ Engine component ~~~
            let gate = circ!(self.engine).add_function_node(component);
            
            // Add port to wire set:
            for (index, &c) in props.ports.iter().enumerate() {
                let port = FunctionPort { gate, index };
                let value = circ!(self.physical).wires.add_port(c, port, || circ!(self.engine).add_value_node())
                    .expect("Expected port addition to be successful");
                
                circ!(self.engine).connect_one(value, port);
            }

            circ!(self.physical).components.insert(gate, props);
        } else {
            // ~~~ UI component ~~~

            // Add tunnel to wire set:
            if !props.label.is_empty() && matches!(props.extra, PhysicalComponentEnum::Tunnel(_)) {
                let &[coord] = props.ports.as_slice() else { unreachable!("Tunnel should have 1 port") };
                let sym = circ!(self.physical).tunnel_interner.add_ref(&props.label);
                circ!(self.physical).wires.add_tunnel(coord, sym, || circ!(self.engine).add_value_node());
            }

            circ!(self.physical).ui_components.insert(props);
        }
        
        Ok(())
    }

    /// Removes a function component from the circuit.
    /// 
    /// This returns [`ReprEditErr::CannotRemoveComponent`] if the component does not exist.
    fn remove_function_component(&mut self, gate: FunctionKey) -> Result<(), ReprEditErr> {
        let props = circ!(self.physical).components.remove(gate)
            .ok_or(ReprEditErr::CannotRemoveComponent)?;

        let result = circ!(self.engine).remove_function_node(gate);
        debug_assert!(result, "Engine removal should succeed");
        
        // Remove all ports from wire set:
        for index in 0..props.ports.len() {
            let port = FunctionPort { gate, index };

            let result = circ!(self.physical).wires.remove_port(port)
                .expect("Component removal should succeed");
            self.handle_remove(result);
        }

        Ok(())
    }

    /// Removes a UI component from the circuit.
    /// 
    /// This returns [`ReprEditErr::CannotRemoveComponent`] if the component does not exist.
    fn remove_ui_component(&mut self, key: UIKey) -> Result<(), ReprEditErr> {
        let props = circ!(self.physical).ui_components.remove(key)
            .ok_or(ReprEditErr::CannotRemoveComponent)?;

        if matches!(props.extra, PhysicalComponentEnum::Tunnel(_)) {
            let sym = circ!(self.physical).tunnel_interner.del_ref(&props.label)
                .expect("Tunnel should have an assigned symbol");
            circ!(self.physical).wires.remove_tunnel(props.origin, sym)
                .expect("Tunnel removal should succeed");
        } else {
            todo!("Non-tunnel UI component removal of wires");
        }

        Ok(())
    }
    /// Removes a component from the circuit.
    /// 
    /// This returns [`ReprEditErr::CannotRemoveComponent`] if the component does not exist.
    pub fn remove_component(&mut self, key: ComponentKey) -> Result<(), ReprEditErr> {
        match key {
            ComponentKey::Function(k) => self.remove_function_component(k),
            ComponentKey::UI(k) => self.remove_ui_component(k),
        }
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
                        wire::MeshKey::Port(p) => Some(p),
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
}