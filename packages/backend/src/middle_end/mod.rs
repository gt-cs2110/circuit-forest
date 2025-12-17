use slotmap::{SecondaryMap, SlotMap, new_key_type};

use crate::circuit::graph::FunctionKey;
use crate::circuit::{CircuitForest, CircuitKey};
use crate::middle_end::wire::{Wire, WireSet};

mod serialize;
pub mod wire;

type Axis = u32;
type Coord = (Axis, Axis);

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
    components: SecondaryMap<FunctionKey, ComponentPos>,
    wires: WireSet,
    ui_components: SlotMap<UIKey, ComponentPos>
}

#[derive(Debug, Default)]
pub struct ComponentPos {
    label: String,
    x: Axis,
    y: Axis
}

pub enum ReprEditErr {
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