mod wiring;
mod muxes;

pub use wiring::*;
pub use muxes::*;

use crate::func::ComponentFn;
use crate::middle_end::{AxisDelta, Coord, CoordDelta};
use enum_dispatch::enum_dispatch;

#[enum_dispatch]
pub trait PhysicalComponent {
    fn engine_component(&self) -> Option<ComponentFn>;
    fn component_name(&self) -> &'static str;
    fn bounds(&self) -> ComponentBounds;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComponentBounds<C = CoordDelta> {
    pub bounds: [C; 2],
    pub ports: Vec<C>
}

impl ComponentBounds {
    fn single_port(width: u32, height: u32) -> Self {
        let origin = (width, height / 2);
        ComponentBounds::new_absolute((width, height), vec![origin])
            .into_relative(origin)
    }

    fn single_port_from_bitsize(bitsize: u8) -> Self {
        const MAX_COLS: u32 = 8;
        
        let bitsize = u32::from(bitsize);
        let n_rows = bitsize.div_ceil(MAX_COLS);
        let height = 2 * n_rows;
        
        match bitsize {
            // If two bits, use a 2 x 2 tile
            ..=2 => Self::single_port(2, height),
            // If 2-8 bits, use a 2n x 2 tile
            w @ ..=MAX_COLS => Self::single_port(2 * w, height),
            // If 9+ bits, use a 16 x h tile
            _ => Self::single_port(2 * MAX_COLS, height)
        }
    }

    pub(crate) fn into_absolute(self, origin: Coord) -> Option<ComponentBounds<Coord>> {
        fn add(p: Coord, delta: CoordDelta) -> Option<Coord> {
            p.0.checked_add_signed(delta.0)
                .zip(p.1.checked_add_signed(delta.1))
        }

        let ComponentBounds { bounds: [b0, b1], ports } = self;
        let bounds = [add(origin, b0)?, add(origin, b1)?];
        let ports = ports.into_iter()
            .map(|delta| add(origin, delta))
            .collect::<Option<_>>()?;
        Some(ComponentBounds { bounds, ports })
    }
}
impl ComponentBounds<Coord> {
    fn new_absolute(dims: Coord, ports: impl IntoIterator<Item=Coord>) -> Self {
        Self {
            bounds: [(0, 0), dims],
            ports: Vec::from_iter(ports)
        }
    }
    pub(crate) fn into_relative(self, origin: Coord) -> ComponentBounds {
        fn sub(p: Coord, q: Coord) -> CoordDelta {
            (p.0.wrapping_sub(q.0) as AxisDelta, p.1.wrapping_sub(q.1) as AxisDelta)
        }

        let ComponentBounds { bounds: [b0, b1], ports } = self;
        let bounds = [
            sub(b0, origin),
            sub(b1, origin)
        ];
        let ports = ports.into_iter()
            .map(|p| sub(p, origin))
            .collect();
        
        ComponentBounds { bounds, ports }
    }
}

#[enum_dispatch(PhysicalComponent)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[allow(missing_docs)]
pub enum PhysicalComponentEnum {
    // Wiring
    Input, Output, Constant, Splitter, Power, Ground, Tunnel,
    // Muxes
    Mux, Demux, Decoder
}