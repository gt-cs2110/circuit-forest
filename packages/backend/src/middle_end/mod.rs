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
    forest: CircuitForest,
    physical: SecondaryMap<CircuitKey, CircuitArea>
}

#[derive(Debug, Default)]
pub struct CircuitArea {
    components: SecondaryMap<FunctionKey, ComponentProps>,
    wires: WireSet,
    ui_components: SlotMap<UIKey, ComponentProps>
}

#[derive(Debug, Default)]
pub struct ComponentProps {
    label: String,
    origin: Coord,
    bounds: [Coord; 2],
    ports: Vec<Coord>,
}
impl ComponentProps {
    pub fn new(label: &str, origin: Coord, bounds: ComponentBounds) -> Option<Self> {
        fn add(p: Coord, delta: CoordDelta) -> Option<Coord> {
            p.0.checked_add_signed(delta.0)
                .zip(p.1.checked_add_signed(delta.1))
        }

        let ComponentBounds { bounds: [b0, b1], ports } = bounds;
        let bounds = [add(origin, b0)?, add(origin, b1)?];
        let ports = ports.into_iter()
            .map(|delta| add(origin, delta))
            .collect::<Option<_>>()?;

        Some(Self {
            label: label.to_string(),
            origin,
            bounds,
            ports
        })
    }
}

pub enum ReprEditErr {
    CannotAddComponent,
    CannotRemoveWire,
    Todo
}
impl MiddleRepr {
    pub fn new() -> Self {
        Self {
            forest: CircuitForest::new(),
            physical: Default::default()
        }
    }

    pub fn add_component(&mut self, ckey: CircuitKey, physical: PhysicalComponentEnum, pos: Coord) -> Result<(), ReprEditErr> {
        let props = ComponentProps::new("", pos, physical.bounds())
            .ok_or(ReprEditErr::CannotAddComponent)?;

        if let Some(component) = physical.engine_component() {
            // Is engine component:
            let fkey = self.forest.circuit(ckey).add_function_node(component);
            self.physical[ckey].components.insert(fkey, props);
        } else {
            // Is UI component:
            self.physical[ckey].ui_components.insert(props);
        }
        
        Ok(())
    }
    
    pub fn add_wire(&mut self, ckey: CircuitKey, w: Wire) -> Result<(), ReprEditErr> {
        // Add to wire set if it doesn't overlap with anything.
        // Cases:
        // - If a wire endpoint connects to the middle of a wire, the wire needs to be split (ValueKey is same)
        // - If a wire connects two wire meshes (e.g., two ValueKey sets), the two ValueKeys must be merged
        
        let [p, q] = w.endpoints();

        let result = self.physical[ckey].wires.add_wire(p, q, || self.forest.circuit(ckey).add_value_node())
            .unwrap_or_else(|| unreachable!("p, q are 1d"));
        match result {
            wire::AddWireResult::NoJoin(_) => {},
            wire::AddWireResult::Join(c, k1, keys) => {
                self.forest.circuit(ckey).join(&keys);
                self.physical[ckey].wires.flood_fill(c, k1);
            },
        }

        Ok(())
    }
    pub fn remove_wire(&mut self, ckey: CircuitKey, w: Wire) -> Result<(), ReprEditErr> {
        let [p, q] = w.endpoints();

        let wire::RemoveWireResult { deleted_keys, split_groups } = self.physical[ckey].wires.remove_wire(p, q)
            .ok_or(ReprEditErr::CannotRemoveWire)?;

        todo!()
    }
}