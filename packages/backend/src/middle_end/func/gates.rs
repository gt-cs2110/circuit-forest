use crate::engine::func;
use crate::middle_end::func::{AbsoluteComponentBounds, PhysicalComponent, PhysicalInitContext, RelativeComponentBounds};

pub use func::GateKind;

/// A gate component with a variable number of inputs.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Gate {
    sim: func::Gate
}
impl PhysicalComponent for Gate {
    fn engine_component(&self) -> Option<func::ComponentFn> {
        Some(self.sim.into())
    }

    fn component_name(&self) -> &'static str {
        self.sim.kind().into()
    }

    fn bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        // The origin is at the output port, which is at (4,2) in absolute coordinates.
        let bounds = [(-4, -2), (0, 2)];
        let inputs = self.sim.n_inputs() as i32;
        
        let mut ports = vec![];
        // Input ports
        // For a 4-input gate, you have (-4, -3), (-4, -1), (-4, 1), (-4, 3)
        // For a 5-input gate, you have (-4, -4), (-4, -2), (-4, 0), (-4, 2), (-4, 4)
        ports.extend((0..inputs).map(|i| (-4, 2 * i - (inputs - 1))));
        ports.push((0, 0)); // Output port

        RelativeComponentBounds { bounds, ports }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
/// A NOT gate component.
pub struct Not {
    sim: func::Not,
}

impl PhysicalComponent for Not {
    fn engine_component(&self) -> Option<func::ComponentFn> {
        Some(self.sim.into())
    }

    fn component_name(&self) -> &'static str {
        "Not"
    }

    fn bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        let origin = (3, 1);
        AbsoluteComponentBounds::new((3, 2), [(0, 1), origin])
            .into_relative(origin)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
/// A tri-state buffer.
pub struct TriState {
    sim: func::TriState,
}

impl PhysicalComponent for TriState {
    fn engine_component(&self) -> Option<func::ComponentFn> {
        Some(self.sim.into())
    }

    fn component_name(&self) -> &'static str {
        "TriState"
    }

    fn bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        let origin = (2, 1);
        AbsoluteComponentBounds::new((2, 2), [(0, 1), (1, 2), origin])
            .into_relative(origin)
    }
}