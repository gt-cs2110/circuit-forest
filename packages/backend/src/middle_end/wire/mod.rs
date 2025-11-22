use std::collections::HashMap;

use crate::circuit::graph::ValueKey;
use crate::middle_end::{Axis, Coord};

#[derive(Debug, Default)]
pub struct Wires {
    wires: HashMap<Wire, ValueKey>
}
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash, serde::Serialize, serde::Deserialize)]
pub struct Wire {
    pub x: Axis,
    pub y: Axis,
    pub length: Axis,
    #[serde(rename = "isHorizontal")]
    pub horizontal: bool
}
impl Wire {
    pub fn from_endpoints(p: Coord, q: Coord) -> Option<Self> {
        // Let p = the left-/top-most coord, q = the other coord.
        let [p, q] = if p <= q { [p, q] } else { [q, p] };

        match (q.0 - p.0, q.1 - p.1) {
            (0, length) => Some(Self { x: p.0, y: p.1, length, horizontal: true }),
            (length, 0) => Some(Self { x: p.0, y: p.1, length, horizontal: false }),
            _ => None
        }
    }

    /// The endpoints of the wire.
    pub fn endpoints(&self) -> [Coord; 2] {
        // FIXME: Handle overflow
        match self.horizontal {
            true  => [(self.x, self.y), (self.x + self.length, self.y)],
            false => [(self.x, self.y), (self.x, self.y + self.length)],
        }
    }

    /// Detect whether this wire includes the specified coordinate.
    pub fn contains(&self, c: Coord) -> bool {
        match self.horizontal {
            true  => self.y == c.1 && self.x <= c.0 && c.0 <= self.x.saturating_add(self.length),
            false => self.x == c.0 && self.y <= c.1 && c.1 <= self.y.saturating_add(self.length),
        }
    }

    pub fn split(&self, c: Coord) -> Option<[Wire; 2]> {
        match self.contains(c) {
            true => {
                let [l, r] = self.endpoints();
                Self::from_endpoints(l, c)
                    .zip(Self::from_endpoints(c, r))
                    .map(Into::into)
            }
            false => None
        }
    }

    pub fn join(&self, w: Wire) -> Option<Wire> {
        // if both are same orientation & align on endpoint then join
        let [l1, r1] = self.endpoints();
        let [l2, r2] = w.endpoints();

        if r1 == l2 {
            Self::from_endpoints(l1, r2)
        } else if l1 == r2 {
            Self::from_endpoints(l2, r1)
        } else {
            None
        }
    }
}