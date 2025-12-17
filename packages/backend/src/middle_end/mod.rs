use slotmap::{SecondaryMap, SlotMap, new_key_type};

use crate::circuit::graph::{FunctionKey, FunctionPort};
use crate::circuit::{CircuitForest, CircuitKey};
use crate::middle_end::func::{ComponentBounds, PhysicalComponent, PhysicalComponentEnum};
use crate::middle_end::wire::{Wire, WireSet};

mod serialize;
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
    ui_components: SlotMap<UIKey, ComponentProps>
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
    pub fn add_component<C: Into<PhysicalComponentEnum>>(&mut self, physical: C, pos: Coord) -> Result<(), ReprEditErr> {
        let physical = physical.into();
        let ComponentBounds { bounds, ports } = physical.bounds().into_absolute(pos)
            .ok_or(ReprEditErr::CannotAddComponent)?;
        let props = ComponentProps {
            label: String::new(),
            origin: pos,
            bounds,
            ports,
            extra: physical,
        };

        if let Some(component) = physical.engine_component() {
            // Is engine component:
            let fkey = circ!(self.engine).add_function_node(component);
            circ!(self.physical).components.insert(fkey, props);
        } else {
            // Is UI component:
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

        let wire::RemoveWireResult { deleted_keys, split_groups } = circ!(self.physical).wires.remove_wire(p, q)
            .ok_or(ReprEditErr::CannotRemoveWire)?;

        for k in deleted_keys {
            circ!(self.engine).remove_value_node(k);
        }
        for (k, groups) in split_groups {
            for group in &groups[1..] {
                let &c = group.iter().next().unwrap();

                // Get all ports associated with coordinates:
                let ports: Vec<_> = circ!(self.physical).components.iter()
                    .flat_map(|(gate, p)| {
                        p.ports.iter()
                            .enumerate()
                            .filter(|&(_, p)| group.contains(p))
                            .map(move |(index, _)| FunctionPort { gate, index })
                    }).collect();

                // Split and update physical:
                let flood_key = circ!(self.engine).split(k, &ports);
                circ!(self.physical).wires.flood_fill(c, flood_key);
            }
        }

        Ok(())
    }

    pub fn run(&mut self) {
        todo!()
    }
}