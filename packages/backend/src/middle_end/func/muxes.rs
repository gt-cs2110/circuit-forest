use crate::engine::func::{self, ComponentFn};
use crate::middle_end::func::{AbsoluteComponentBounds, Handedness, Orientation, PhysicalComponent, PhysicalInitContext, RelativeComponentBounds};

const PLEXER_WIDTH: u32 = 3;

/// A multiplexer (mux) component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mux {
    bitsize: u8,
    selsize: u8,
    orientation: Orientation,
    handedness: Handedness
}
impl PhysicalComponent for Mux {
    fn init_engine(&self) -> Option<ComponentFn> {
        Some(func::Mux::new(self.bitsize, self.selsize).into())
    }

    fn component_name(&self) ->  &'static str {
        "Mux"
    }

    fn init_bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        let n_inputs: u32 = func::Mux::n_inputs_from(self.selsize) as u32;
        
        let width = PLEXER_WIDTH;
        let height = 2 * n_inputs;

        let origin = (width, n_inputs);

        let mut ports = vec![(1, height)]; // selector
        ports.extend((0..n_inputs).map(|i| (0, 1 + 2*i))); // inputs
        ports.push(origin); // output

        AbsoluteComponentBounds::new((width, height), ports)
            .into_relative(origin)
            .orient(self.orientation, self.handedness)
    }
}

/// A demultiplexer (demux) component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Demux {
    bitsize: u8,
    selsize: u8,
    orientation: Orientation,
    handedness: Handedness
}
impl PhysicalComponent for Demux {
    fn init_engine(&self) -> Option<ComponentFn> {
        Some(func::Demux::new(self.bitsize, self.selsize).into())
    }

    fn component_name(&self) ->  &'static str {
        "Demux"
    }

    fn init_bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        let n_outputs = func::Demux::n_outputs_from(self.selsize) as u32;
        
        let width = PLEXER_WIDTH;
        let height = 2 * n_outputs;

        let origin = (0, n_outputs);
        let mut ports = vec![(1, height), origin]; // selector, input
        ports.extend((0..n_outputs).map(|i| (width, 1 + 2*i))); // outputs

        AbsoluteComponentBounds::new((width, height), ports)
            .into_relative(origin)
            .orient(self.orientation, self.handedness)
    }
}

/// A decoder component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Decoder {
    selsize: u8,
    orientation: Orientation,
    handedness: Handedness
}
impl PhysicalComponent for Decoder {
    fn init_engine(&self) -> Option<ComponentFn> {
        Some(func::Decoder::new(self.selsize).into())
    }

    fn component_name(&self) ->  &'static str {
        "Decoder"
    }

    fn init_bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        let n_outputs = func::Decoder::n_outputs_from(self.selsize) as u32;
        
        let width = PLEXER_WIDTH;
        let height = 2 * n_outputs;

        let origin = (1, height);
        let mut ports = vec![origin]; // selector
        ports.extend((0..n_outputs).map(|i| (width, 1 + 2*i))); // outputs

        AbsoluteComponentBounds::new((width, height), ports)
            .into_relative(origin)
            .orient(self.orientation, self.handedness)
    }
}
