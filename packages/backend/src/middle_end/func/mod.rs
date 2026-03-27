//! Logic components for middle-end circuits.

mod wiring;
mod muxes;
mod misc;
mod gates;

pub use wiring::*;
pub use muxes::*;
pub use misc::*;
pub use gates::*;

use crate::engine::func::ComponentFn;
use crate::middle_end::{AxisDelta, Coord, CoordDelta, MiddleCircuit};
use enum_dispatch::enum_dispatch;

/// Helper which rotates a coordinate around the origin to match the provided orientation.
/// 
/// By default, components are defined as having east orientation with down-right handedness.
/// This function assumes this port is on an east-oriented component and
/// rotates it to match what it should be on a component with specified `orientation` and `handedness`.
/// 
/// If you only wish to rotate, you can specify handedness with `Default::default()` [down-right handedness].
fn orient_coord(c: CoordDelta, orientation: Orientation, handedness: Handedness) -> CoordDelta {
    let (x, y) = c;
    let y = match handedness {
        Handedness::TopLeft   => -y,
        Handedness::DownRight => y,
    };
    match orientation {
        // To transform east to north, we rotate 90 deg CCW,
        // which transforms (x, y) to (-y, x)
        Orientation::North => (-y,  x),
        Orientation::East  => ( x,  y),
        Orientation::South => ( y, -x),
        Orientation::West  => (-x, -y)
    }
}

/// Orientation.
/// 
/// This is typically used to describe the orientation of a component which can be rotated.
#[expect(missing_docs)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Orientation {
    North, South, #[default] East, West
}

/// The handedness (or mirror orientation).
/// 
/// This is typically used to describe the mirror orientation of
/// a chiral component (one which is not mirror-symmetric).
/// 
/// For a chiral component, there is at least one port which is not symmetric
/// across the component's main axis of symmetry.
/// 
/// For example, the selector port for muxes and decoders, or the join port for splitters.
/// 
/// We will refer to this as the "chiral" port.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Handedness {
    /// Left-handedness.
    /// 
    /// For east-oriented components, the chiral port is pointed northwards (top).
    /// For north-oriented components, the chiral port is pointed westwards (left).
    TopLeft,

    /// Right-handedness.
    /// 
    /// For east-oriented components, the chiral port is pointed southwards (down).
    /// For north-oriented components, the chiral port is pointed eastwards (right).
    #[default]
    DownRight
}

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
    /// Initializes the component which represents the engine logic of this physical component,
    /// based on the properties of the physical component.
    /// 
    /// This can be `None` if this component has no engine logic.
    fn init_engine(&self) -> Option<ComponentFn>;

    /// The name of the component.
    fn component_name(&self) -> &'static str;

    /// Initializes the bounds of the physical component,
    /// based on its properties.
    /// 
    /// The bounds are defined as:
    ///   - The area encompassed of the component
    ///     (defined by the top-leftmost point and the bottom-rightmost point)
    ///   - The position of the ports
    /// 
    /// These components are relative to the origin (0, 0),
    /// meaning that when placed, the locations are relative
    /// to the point the component is placed.
    fn init_bounds(&self, ctx: PhysicalInitContext<'_>) -> RelativeComponentBounds;
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

    /// Orients the component bounds and ports around the origin
    /// in accordance to the specified orientation and handedness,
    /// assuming the coordinates were originally oriented eastwards with down-right handedness.
    pub fn orient(self, orientation: Orientation, handedness: Handedness) -> Self {
        // Rotate bounds and ports, then get use the max and min of the corners to get new bounds:
        let Self { bounds: [b0, b1], ports } = self;
        // The bounds are defined by min and max x and y coordinates, but when we rotate,
        // the new bounds may be defined by different corners, so we need to consider all corners to get the new bounds.
        let corners = [
            b0,
            (b0.0, b1.1),
            (b1.0, b0.1),
            b1,
        ].map(|c| orient_coord(c, orientation, handedness));

        let min_x = corners.iter().map(|c| c.0).min().unwrap();
        let max_x = corners.iter().map(|c| c.0).max().unwrap();
        let min_y = corners.iter().map(|c| c.1).min().unwrap();
        let max_y = corners.iter().map(|c| c.1).max().unwrap();
        let bounds = [(min_x, min_y), (max_x, max_y)];


        // Rotate ports:
        let ports = ports.into_iter()
            .map(|p| orient_coord(p, orientation, handedness))
            .collect(); 
        
        Self { bounds, ports }
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
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, strum::EnumDiscriminants, strum::IntoStaticStr)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize), serde(untagged))]
#[expect(missing_docs)]
#[strum_discriminants(
    name(PhysicalComponentKind),
    expect(missing_docs),
    derive(strum::IntoStaticStr)
)]
pub enum PhysicalComponentEnum {
    // Wiring
    Pin, Constant, Splitter, Power, Ground, Tunnel, Probe,
    // Muxes
    Mux, Demux, Decoder,
    // Misc
    Text, Subcircuit,
    //Gates
    Gate, Not, TriState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orient_ports_by_direction() {
        let base = RelativeComponentBounds {
            bounds: [(-2, -1), (3, 4)],
            ports: vec![(-1, 0), (0, 1), (2, -3)]
        };

        let east = base.clone().orient(Orientation::East, Handedness::DownRight);
        assert_eq!(east.ports, vec![(-1, 0), (0, 1), (2, -3)]);

        let north = base.clone().orient(Orientation::North, Handedness::DownRight);
        assert_eq!(north.ports, vec![(0, -1), (-1, 0), (3, 2)]);

        let south = base.clone().orient(Orientation::South, Handedness::DownRight);
        assert_eq!(south.ports, vec![(0, 1), (1, 0), (-3, -2)]);

        let west = base.clone().orient(Orientation::West, Handedness::DownRight);
        assert_eq!(west.ports, vec![(1, 0), (0, -1), (-2, 3)]);

        let east = base.clone().orient(Orientation::East, Handedness::TopLeft);
        assert_eq!(east.ports, vec![(-1, 0), (0, 1), (2, -3)]);

        let north = base.clone().orient(Orientation::North, Handedness::TopLeft);
        assert_eq!(north.ports, vec![(0, -1), (-1, 0), (3, 2)]);

        let south = base.clone().orient(Orientation::South, Handedness::TopLeft);
        assert_eq!(south.ports, vec![(0, 1), (1, 0), (-3, -2)]);

        let west = base.orient(Orientation::West, Handedness::TopLeft);
        assert_eq!(west.ports, vec![(1, 0), (0, -1), (-2, 3)]);
    }

    #[test]
    fn orient_bounds_accounts_for_all_corners() {
        let base = RelativeComponentBounds {
            bounds: [(-2, -1), (3, 4)],
            ports: vec![(-1, 0)]
        };
        let [b0, b1] = base.bounds;
        let corners = [b0, (b0.0, b1.1), (b1.0, b0.1), b1];

        for orientation in [
            Orientation::North,
            Orientation::East,
            Orientation::South,
            Orientation::West,
        ] {
            for handedness in [Handedness::TopLeft, Handedness::DownRight] {
                let oriented = base.clone().orient(orientation, handedness);
                let [lo, hi] = oriented.bounds;
    
                // Bounds should stay normalized.
                assert!(lo.0 <= hi.0);
                assert!(lo.1 <= hi.1);
    
                // Every rotated corner must lie inside the oriented bounds.
                for c in corners {
                    let r = orient_coord(c, orientation, handedness);
                    assert!(r.0 >= lo.0 && r.0 <= hi.0);
                    assert!(r.1 >= lo.1 && r.1 <= hi.1);
                }
            }
        }
    }
}
