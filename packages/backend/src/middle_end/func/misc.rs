use crate::engine::{CircuitKey, func};
use crate::middle_end::func::{PhysicalComponent, PhysicalInitContext, RelativeComponentBounds};

/// A subcircuit component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Subcircuit {
    key: CircuitKey
}
impl PhysicalComponent for Subcircuit {
    fn init_engine(&self) -> Option<func::ComponentFn> {
        Some(func::Subcircuit::new(self.key).into())
    }

    fn component_name(&self) ->  &'static str {
        "Subcircuit"
    }

    fn init_bounds(&self, ctx: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        todo!()
    }
}

/// Text.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Text;
impl PhysicalComponent for Text {
    fn init_engine(&self) -> Option<func::ComponentFn> {
        None
    }

    fn component_name(&self) ->  &'static str {
        "Text"
    }

    fn init_bounds(&self, ctx: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        todo!()
    }
}