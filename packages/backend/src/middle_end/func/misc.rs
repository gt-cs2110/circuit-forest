use crate::engine::func;
use crate::engine::CircuitKey;
use crate::middle_end::func::{PhysicalComponent, PhysicalInitContext, RelativeComponentBounds};

/// A subcircuit component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Subcircuit {
    sim: func::Subcircuit
}
impl Subcircuit {
    /// Creates a new instance of the subcircuit with specified circuit key.
    pub fn new(key: CircuitKey) -> Self {
        Self { sim: func::Subcircuit::new(key) }
    }
}
impl PhysicalComponent for Subcircuit {
    fn engine_component(&self) -> Option<func::ComponentFn> {
        Some(self.sim.into())
    }

    fn component_name(&self) ->  &'static str {
        "Subcircuit"
    }

    fn bounds(&self, ctx: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        todo!()
    }
}

/// Text.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Text;
impl Text {
    /// Creates a new instance of text.
    pub fn new() -> Self {
        Self
    }
}
impl PhysicalComponent for Text {
    fn engine_component(&self) -> Option<func::ComponentFn> {
        None
    }

    fn component_name(&self) ->  &'static str {
        "Text"
    }

    fn bounds(&self, ctx: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        todo!()
    }
}
