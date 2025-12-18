use slotmap::{SecondaryMap, SlotMap, new_key_type};

use crate::circuit::graph::{FunctionKey, FunctionPort};
use crate::circuit::{CircuitForest, CircuitKey};
use crate::middle_end::func::{ComponentBounds, PhysicalComponent, PhysicalComponentEnum};
use crate::middle_end::string_interner::StringInterner;
use crate::middle_end::wire::{Wire, WireSet};

mod serialize;
mod string_interner;
pub mod wire;
pub mod func;

type Axis = u32;
type Coord = (Axis, Axis);

type AxisDelta = i32;
type CoordDelta = (AxisDelta, AxisDelta);

new_key_type! {
    /// Key for UI components that are not part of component.
    pub struct UIKey;
}

#[derive(Debug, Default)]
pub struct MiddleRepr {
    engine: CircuitForest,
    physical: SecondaryMap<CircuitKey, CircuitArea>
}

#[derive(Debug, Default)]
pub struct CircuitArea {
    components: SecondaryMap<FunctionKey, ComponentProps>,
    wires: WireSet,
    ui_components: SlotMap<UIKey, ComponentProps>,
    tunnel_interner: StringInterner
}

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

pub enum ReprEditErr {
    CannotAddComponent,
    CannotAddWire,
    CannotRemoveWire,
    Todo
}
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
    pub fn add_component<C: Into<PhysicalComponentEnum>>(&mut self, physical: C, label: &str, pos: Coord) -> Result<(), ReprEditErr> {
        let physical = physical.into();
        let ComponentBounds { bounds, ports } = physical.bounds().into_absolute(pos)
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
                let result = circ!(self.physical).wires.add_port(c, port, || circ!(self.engine).add_value_node());
                debug_assert!(result.is_some(), "Expected port addition to be successful");
            }

            circ!(self.physical).components.insert(gate, props);
        } else {
            // ~~~ UI component ~~~

            // Add tunnel to wire set:
            if !props.label.is_empty() && matches!(props.extra, PhysicalComponentEnum::Tunnel(_)) {
                let &[coord] = props.ports.as_slice() else { unreachable!("Tunnel should have 1 port") };
                let sym = circ!(self.physical).tunnel_interner.intern(&props.label);
                circ!(self.physical).wires.add_tunnel(coord, sym, || circ!(self.engine).add_value_node());
            }

            circ!(self.physical).ui_components.insert(props);
        }
        
        Ok(())
    }
    pub fn remove_component(&mut self, key: FunctionKey) -> Result<(), ReprEditErr> {
        todo!()
    }

    pub fn add_wire(&mut self, w: Wire) -> Result<(), ReprEditErr> {
        // Add to wire set if it doesn't overlap with anything.
        // Cases:
        // - If a wire endpoint connects to the middle of a wire, the wire needs to be split (ValueKey is same)
        // - If a wire connects two wire meshes (e.g., two ValueKey sets), the two ValueKeys must be merged
        
        let [p, q] = w.endpoints();

        let result = circ!(self.physical).wires.add_wire(p, q, || circ!(self.engine).add_value_node())
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
    pub fn remove_wire(&mut self, w: Wire) -> Result<(), ReprEditErr> {
        let [p, q] = w.endpoints();

        let result = circ!(self.physical).wires.remove_wire(p, q)
            .ok_or(ReprEditErr::CannotRemoveWire)?;

        self.resolve_remove(result);

        Ok(())
    }

    /// Updates engine to corresponding `RemoveWireResult`.
    fn resolve_remove(&mut self, result: wire::RemoveWireResult) {
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

    pub fn run(&mut self) {
        todo!()
    }
}