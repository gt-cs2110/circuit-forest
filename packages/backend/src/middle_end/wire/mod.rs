mod range_map;
mod wire_set;

use std::num::NonZero;

use crate::middle_end::{Axis, Coord};

pub use range_map::WireRangeMap;
pub use wire_set::{WireSet, AddWireResult, RemoveWireResult};

fn minmax<T: Ord>(p: T, q: T) -> [T; 2] {
    if q < p { [q, p] } else { [p, q] }
}

/// A wire.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, serde::Serialize, serde::Deserialize)]
pub struct Wire {
    /// The lowermost X coordinate of the wire.
    x: Axis,
    /// The lowermost Y coordinate of the wire.
    y: Axis,
    /// The length of the wire.
    length: NonZero<Axis>,
    /// Whether the wire is horizontal or vertical.
    #[serde(rename = "isHorizontal")]
    horizontal: bool
}
impl Wire {
    /// Creates a new Wire, returning None if `length` would result in overflowing coordinates.
    pub fn new(x: Axis, y: Axis, length: Axis, horizontal: bool) -> Option<Self> {
        let acceptable = match horizontal {
            true  => x.checked_add(length).is_some(),
            false => y.checked_add(length).is_some()
        };

        acceptable.then_some(Self { x, y, length: NonZero::new(length)?, horizontal })
    }
    /// Returns a wire from parameters, assuming all conditions are correct.
    /// 
    /// Panics if not.
    fn new_raw(x: Axis, y: Axis, length: NonZero<Axis>, horizontal: bool) -> Self {
        let _ = x.strict_add(length.get());
        let _ = y.strict_add(length.get());
        Self { x, y, length, horizontal }
    }

    /// Constructs a wire out of endpoints, returning None if not 1D.
    pub fn from_endpoints(p: Coord, q: Coord) -> Option<Self> {
        // Let p = the left-/top-most coord, q = the other coord.
        let [p, q] = minmax(p, q);

        match (NonZero::new(q.0 - p.0), NonZero::new(q.1 - p.1)) {
            (None, None) => None,
            (None, Some(length)) => Some(Self { x: p.0, y: p.1, length, horizontal: false }),
            (Some(length), None) => Some(Self { x: p.0, y: p.1, length, horizontal: true }),
            _ => None
        }
    }

    /// The endpoints of the wire.
    pub fn endpoints(&self) -> [Coord; 2] {
        match self.horizontal {
            true  => [(self.x, self.y), (self.x + self.length.get(), self.y)],
            false => [(self.x, self.y), (self.x, self.y + self.length.get())],
        }
    }

    /// Detect whether this wire includes the specified coordinate.
    pub fn contains(&self, c: Coord) -> bool {
        match self.horizontal {
            true  => self.y == c.1 && (self.x <= c.0) && (c.0 - self.x <= self.length.get()),
            false => self.x == c.0 && (self.y <= c.1) && (c.1 - self.y <= self.length.get()),
        }
    }

    /// Creates an iterable of every coordinate in the wire.
    pub fn coord_iter(&self) -> impl DoubleEndedIterator<Item=Coord> {
        let range = match self.horizontal {
            true  => self.x ..= self.x + self.length.get(),
            false => self.y ..= self.y + self.length.get(),
        };
        let mapper = |c| match self.horizontal {
            true  => (c, self.y),
            false => (self.x, c),
        };
        range.map(mapper)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_new() {
        assert!(Wire::new(1, 1, 10, true).is_some());
        assert!(Wire::new(1, 1, 10, false).is_some());

        // edge cases
        assert!(Wire::new(1, 9, Axis::MAX - 8, true).is_some());
        assert!(Wire::new(1, 9, Axis::MAX - 8, false).is_none());
        assert!(Wire::new(1, 1, 0, true).is_none());
    }
    #[test]
    fn wire_from_endpoints() {
        // Horizontal wire
        let [p, q] = [(1, 4), (5, 4)];
        let wire = Wire::new(1, 4, 4, true).unwrap();
        
        assert_eq!(Wire::from_endpoints(p, q), Some(wire));
        assert_eq!(Wire::from_endpoints(q, p), Some(wire));
        
        // Vertical wire
        let [p, q] = [(1, 2), (1, 9)];
        let wire = Wire::new(1, 2, 7, false).unwrap();

        assert_eq!(Wire::from_endpoints(p, q), Some(wire));
        assert_eq!(Wire::from_endpoints(q, p), Some(wire));

        // Diagonal wires
        let [p, q] = [(1, 2), (3, 4)];
        assert_eq!(Wire::from_endpoints(p, q), None);
        assert_eq!(Wire::from_endpoints(q, p), None);

        // Zero wires
        assert_eq!(Wire::from_endpoints(p, p), None);
    }
    #[test]
    fn wire_endpoints() {
        // Horizontal wire
        let [p, q] = [(1, 4), (5, 4)];
        let w = Wire::from_endpoints(p, q).unwrap();
        assert_eq!(w.endpoints(), [p, q]);
        let w = Wire::from_endpoints(q, p).unwrap();
        assert_eq!(w.endpoints(), [p, q]);
        
        let [p, q] = [(1, 2), (1, 9)];
        let w = Wire::from_endpoints(p, q).unwrap();
        assert_eq!(w.endpoints(), [p, q]);
        let w = Wire::from_endpoints(q, p).unwrap();
        assert_eq!(w.endpoints(), [p, q]);
    }

    #[test]
    fn wire_contains() {
        // Horizontal wire
        let [p, q] = [(1, 4), (5, 4)];
        let w = Wire::from_endpoints(p, q).unwrap();
        assert!(!w.contains((0, 4)));
        assert!(w.contains((1, 4)));
        assert!(w.contains((2, 4)));
        assert!(w.contains((3, 4)));
        assert!(w.contains((4, 4)));
        assert!(w.contains((5, 4)));
        assert!(!w.contains((6, 4)));
        assert!(!w.contains((1, 5)));
        
        // Vertical wire
        let [p, q] = [(1, 2), (1, 6)];
        let w = Wire::from_endpoints(p, q).unwrap();
        assert!(!w.contains((1, 1)));
        assert!(w.contains((1, 2)));
        assert!(w.contains((1, 3)));
        assert!(w.contains((1, 4)));
        assert!(w.contains((1, 5)));
        assert!(w.contains((1, 6)));
        assert!(!w.contains((1, 7)));
        assert!(!w.contains((0, 0)));
    }
}