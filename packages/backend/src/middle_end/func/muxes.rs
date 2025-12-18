use crate::func::{self, ComponentFn};
use crate::middle_end::func::{AbsoluteComponentBounds, PhysicalComponent, RelativeComponentBounds};

const PLEXER_WIDTH: u32 = 3;

/// A multiplexer (mux) component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Mux {
    sim: func::Mux
}
impl PhysicalComponent for Mux {
    fn engine_component(&self) -> Option<ComponentFn> {
        Some(self.sim.into())
    }

    fn component_name(&self) ->  &'static str {
        "Mux"
    }

    fn bounds(&self) -> RelativeComponentBounds {
        let n_inputs = self.sim.n_inputs() as u32;
        
        let width = PLEXER_WIDTH;
        let height = 2 * n_inputs;

        let origin = (width, n_inputs);
        let ports = [(1, height)].into_iter() // selector
            .chain((0..n_inputs).map(|i| (0, 1 + i))) // inputs
            .chain([origin]); //output

        AbsoluteComponentBounds::new((width, height), ports)
            .into_relative(origin)
    }
}

/// A demultiplexer (demux) component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Demux {
    sim: func::Demux
}
impl PhysicalComponent for Demux {
    fn engine_component(&self) -> Option<ComponentFn> {
        Some(self.sim.into())
    }

    fn component_name(&self) ->  &'static str {
        "Demux"
    }

    fn bounds(&self) -> RelativeComponentBounds {
        let n_outputs = self.sim.n_outputs() as u32;
        
        let width = PLEXER_WIDTH;
        let height = 2 * n_outputs;

        let origin = (0, n_outputs);
        let ports = [(1, height)].into_iter() // selector
            .chain([origin]) // input
            .chain((0..n_outputs).map(|i| (width, 1 + i))); // outputs

        AbsoluteComponentBounds::new((width, height), ports)
            .into_relative(origin)
    }
}

/// A decoder component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Decoder {
    sim: func::Decoder
}
impl PhysicalComponent for Decoder {
    fn engine_component(&self) -> Option<ComponentFn> {
        Some(self.sim.into())
    }

    fn component_name(&self) ->  &'static str {
        "Decoder"
    }

    fn bounds(&self) -> RelativeComponentBounds {
        let n_outputs = self.sim.n_outputs() as u32;
        
        let width = PLEXER_WIDTH;
        let height = 2 * n_outputs;

        let origin = (1, height);
        let ports = [origin].into_iter() // selector
            .chain((0..n_outputs).map(|i| (width, 1 + i))); // outputs

        AbsoluteComponentBounds::new((width, height), ports)
            .into_relative(origin)
    }
}
