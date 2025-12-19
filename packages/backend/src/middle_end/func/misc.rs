use crate::func;
use crate::middle_end::func::{PhysicalComponent, RelativeComponentBounds};

/// A subcircuit component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Subcircuit {
    sim: func::Subcircuit
}
impl PhysicalComponent for Subcircuit {
    fn engine_component(&self) -> Option<func::ComponentFn> {
        Some(self.sim.into())
    }

    fn component_name(&self) ->  &'static str {
        "Subcircuit"
    }

    fn bounds(&self) -> RelativeComponentBounds {
        todo!()
    }
}

/// Text.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Text;
impl PhysicalComponent for Text {
    fn engine_component(&self) -> Option<func::ComponentFn> {
        None
    }

    fn component_name(&self) ->  &'static str {
        "Text"
    }

    fn bounds(&self) -> RelativeComponentBounds {
        todo!()
    }
}