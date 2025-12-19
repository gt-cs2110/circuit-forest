mod wiring;
mod muxes;
mod misc;

pub use wiring::*;
pub use muxes::*;
pub use misc::*;

use crate::engine::func::ComponentFn;
use crate::middle_end::{AxisDelta, Coord, CoordDelta, MiddleCircuit};
use enum_dispatch::enum_dispatch;

/// Orientation.
/// 
/// This is typically used to describe the orientation of a component which can be rotated.
#[expect(missing_docs)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
pub enum Orientation {
    North, South, East, West
}
/// The handedness (or mirror orientation).
/// 
/// This is typically used to describe the mirror orientation of a component
/// which is chiral (not mirror-symmetric).
/// 
/// This typically affects the position of the main port
/// (e.g., selector port for muxes and decoders, or the join port of a splitter).
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
pub enum Handedness {
    /// Right-handedness. The main port will either be top or left.
    TopLeft,
    /// Left-handedness. The main port will either be down or right.
    DownRight
}

// TODO: Utilize above structs for various physical component definitions

/// Context available during [`PhysicalComponent`] initialization.
pub struct PhysicalInitContext<'a> {
    /// The circuit this component is being placed in.
    pub circuit: &'a MiddleCircuit<'a>,
    /// The label of the component.
    pub label: &'a str
}

/// A component that can be added in a [middle-end circuit](`crate::middle_end::MiddleRepr`).
#[enum_dispatch]
pub trait PhysicalComponent {
    /// A component which represents the engine logic of this component.
    /// 
    /// This can be `None` if this component has no engine logic.
    fn engine_component(&self) -> Option<ComponentFn>;

    /// The name of the component.
    fn component_name(&self) -> &'static str;

    /// The area taken by this component, which includes:
    ///   - The bounds of the component
    ///   - The position of the ports
    /// 
    /// These components are relative to the origin (0, 0),
    /// meaning that when placed, the locations are relative
    /// to the point the component is placed.
    fn bounds(&self, ctx: PhysicalInitContext<'_>) -> RelativeComponentBounds;
}

/// Struct containing the physical bounds of a component and location of ports.
/// 
/// This has two forms:
/// - [`RelativeComponentBounds`]: Bounds with coordinates relative to the origin (0, 0)
/// - [`AbsoluteComponentBounds`]: Bounds with physical coordinates
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComponentBounds<C> {
    /// The bounds (the left-bottom-most point and the right-top-most point)
    pub bounds: [C; 2],
    /// The location of each port.
    pub ports: Vec<C>
}
/// Component bounds with positions relative to the origin (0, 0).
pub type RelativeComponentBounds = ComponentBounds<CoordDelta>;
/// Component bounds with absolute physical positions.
pub type AbsoluteComponentBounds = ComponentBounds<Coord>;

impl<C: Default> ComponentBounds<C> {
    /// Creates a new [`ComponentBounds`].
    pub fn new(dims: C, ports: impl IntoIterator<Item = C>) -> Self {
        Self {
            bounds: [Default::default(), dims],
            ports: Vec::from_iter(ports)
        }
    }
}
impl RelativeComponentBounds {
    fn single_port(width: u32, height: u32) -> Self {
        Self::single_port_with_origin(width, height, (width, height / 2))
    }

    fn single_port_with_origin(width: u32, height: u32, origin: Coord) -> Self {
        ComponentBounds::new((width, height), [origin])
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

    pub(crate) fn into_absolute(self, origin: Coord) -> Option<AbsoluteComponentBounds> {
        fn add(p: Coord, delta: CoordDelta) -> Option<Coord> {
            p.0.checked_add_signed(delta.0)
                .zip(p.1.checked_add_signed(delta.1))
        }

        let Self { bounds: [b0, b1], ports } = self;
        let bounds = [add(origin, b0)?, add(origin, b1)?];
        let ports = ports.into_iter()
            .map(|delta| add(origin, delta))
            .collect::<Option<_>>()?;
        Some(AbsoluteComponentBounds { bounds, ports })
    }
}
impl AbsoluteComponentBounds {
    pub(crate) fn into_relative(self, origin: Coord) -> RelativeComponentBounds {
        fn sub(p: Coord, q: Coord) -> CoordDelta {
            (p.0.wrapping_sub(q.0) as AxisDelta, p.1.wrapping_sub(q.1) as AxisDelta)
        }

        let Self { bounds: [b0, b1], ports } = self;
        let bounds = [
            sub(b0, origin),
            sub(b1, origin)
        ];
        let ports = ports.into_iter()
            .map(|p| sub(p, origin))
            .collect();
        
        RelativeComponentBounds { bounds, ports }
    }
}

#[enum_dispatch(PhysicalComponent)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[allow(missing_docs)]
pub enum PhysicalComponentEnum {
    // Wiring
    Input, Output, Constant, Splitter, Power, Ground, Tunnel, Probe,
    // Muxes
    Mux, Demux, Decoder,
    // Misc
    Text, Subcircuit,
}