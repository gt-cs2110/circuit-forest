use crate::engine::func::{self, ComponentFn};
use crate::middle_end::func::{AbsoluteComponentBounds, PhysicalComponent, PhysicalInitContext, RelativeComponentBounds};

const PLEXER_WIDTH: u32 = 3;

/// A multiplexer (mux) component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Mux {
    sim: func::Mux
}
impl Mux {
    /// Creates a new instance of the mux with specified bitsize and selector size.
    pub fn new(bitsize: u8, selsize: u8) -> Self {
        Self { sim: func::Mux::new(bitsize, selsize) }
    }
}
impl PhysicalComponent for Mux {
    fn engine_component(&self) -> Option<ComponentFn> {
        Some(self.sim.into())
    }

    fn component_name(&self) ->  &'static str {
        "Mux"
    }

    fn bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        let n_inputs: u32 = self.sim.n_inputs() as u32;
        
        let width = PLEXER_WIDTH;
        let height = 2 * n_inputs;

        let origin = (width, n_inputs);

        let mut ports = vec![(1, height)]; // selector
        ports.extend((0..n_inputs).map(|i| (0, 1 + 2*i))); // inputs
        ports.push(origin); // output

        AbsoluteComponentBounds::new((width, height), ports)
            .into_relative(origin)
    }
}

/// A demultiplexer (demux) component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Demux {
    sim: func::Demux
}
impl Demux {
    /// Creates a new instance of the demux with specified bitsize and selector size.
    pub fn new(bitsize: u8, selsize: u8) -> Self {
        Self { sim: func::Demux::new(bitsize, selsize) }
    }
}
impl PhysicalComponent for Demux {
    fn engine_component(&self) -> Option<ComponentFn> {
        Some(self.sim.into())
    }

    fn component_name(&self) ->  &'static str {
        "Demux"
    }

    fn bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        let n_outputs = self.sim.n_outputs() as u32;
        
        let width = PLEXER_WIDTH;
        let height = 2 * n_outputs;

        let origin = (0, n_outputs);
        let mut ports = vec![(1, height), origin]; // selector, input
        ports.extend((0..n_outputs).map(|i| (width, 1 + 2*i))); // outputs

        AbsoluteComponentBounds::new((width, height), ports)
            .into_relative(origin)
    }
}

/// A decoder component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Decoder {
    sim: func::Decoder
}
impl Decoder {
    /// Creates a new instance of the decoder with specified selector size.
    pub fn new(selsize: u8) -> Self {
        Self { sim: func::Decoder::new(selsize) }
    }
}
impl PhysicalComponent for Decoder {
    fn engine_component(&self) -> Option<ComponentFn> {
        Some(self.sim.into())
    }

    fn component_name(&self) ->  &'static str {
        "Decoder"
    }

    fn bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        let n_outputs = self.sim.n_outputs() as u32;
        
        let width = PLEXER_WIDTH;
        let height = 2 * n_outputs;

        let origin = (1, height);
        let mut ports = vec![origin]; // selector
        ports.extend((0..n_outputs).map(|i| (width, 1 + 2*i))); // outputs

        AbsoluteComponentBounds::new((width, height), ports)
            .into_relative(origin)
    }
}
