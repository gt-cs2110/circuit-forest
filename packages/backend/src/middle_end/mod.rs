use slotmap::{SecondaryMap, SlotMap, new_key_type};

use crate::circuit::graph::FunctionKey;
use crate::circuit::{CircuitForest, CircuitKey};
use crate::middle_end::wire::{Wire, Wires};

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
    wires: Wires,
    ui_components: SlotMap<UIKey, ComponentPos>
}

#[derive(Debug, Default)]
pub struct ComponentPos {
    label: String,
    x: Axis,
    y: Axis
}

pub enum ReprEditErr {
    Todo
}
impl MiddleRepr {
    pub fn new() -> Self {
        Self {
            forest: CircuitForest::new(),
            physical: Default::default()
        }
    }

    pub fn add_wire(&mut self, w: Wire) -> Result<(), ReprEditErr> {
        // Add to wire set if it doesn't overlap with anything.
        // Cases:
        // - If a wire endpoint connects to the middle of a wire, the wire needs to be split (ValueKey is same)
        // - If a wire connects two wire meshes (e.g., two ValueKey sets), the two ValueKeys must be merged
        todo!()
    }
    pub fn remove_wire(&mut self, w: Wire) -> Result<(), ReprEditErr> {
        todo!()
    }    
}