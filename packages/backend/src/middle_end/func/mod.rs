mod wiring;
mod muxes;

pub use wiring::*;
pub use muxes::*;

use crate::func::ComponentFn;
use crate::middle_end::CoordDelta;
use enum_dispatch::enum_dispatch;

#[enum_dispatch]
pub trait PhysicalComponent {
    fn engine_component(&self) -> Option<ComponentFn>;
    fn component_name(&self) -> &'static str;
    fn bounds(&self) -> ComponentBounds;
}

#[derive(Debug)]
pub struct ComponentBounds {
    pub bounds: [CoordDelta; 2],
    pub ports: Vec<CoordDelta>
}

impl ComponentBounds {
    fn single_port(width: i32, height: i32) -> Self {
        let half_height = height / 2;
        Self {
            bounds: [(-width, -half_height), (0, height - half_height)],
            ports: vec![(0, 0)],
        }
    }

    fn single_port_from_bitsize(bitsize: u8) -> Self {
        const MAX_COLS: u8 = 8;
        
        let n_rows = i32::from(bitsize.div_ceil(MAX_COLS));
        let height = 2 * n_rows;
        
        match bitsize {
            // If two bits, use a 2 x 2 tile
            ..=2 => Self::single_port(2, height),
            // If 2-8 bits, use a 2n x 2 tile
            w @ ..=MAX_COLS => Self::single_port(2 * i32::from(w), height),
            // If 9+ bits, use a 16 x h tile
            _ => Self::single_port(2 * i32::from(MAX_COLS), height)
        }
    }
}

#[enum_dispatch(PhysicalComponent)]
pub enum PhysicalComponentEnum {
    // Wiring
    Input, Output, Constant, Splitter
}