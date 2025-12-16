use std::collections::BTreeMap;
use std::num::NonZero;

use crate::middle_end::Axis;
use crate::middle_end::wire::Wire;

#[derive(PartialEq, Eq, Clone, Copy, Default)]
pub enum WireAtResult {
    #[default]
    None,
    One(Wire1D),
    Two([Wire1D; 2])
}
impl Iterator for WireAtResult {
    type Item = Wire1D;

    fn next(&mut self) -> Option<Self::Item> {
        let (result, state) = match *self {
            WireAtResult::None => (None, WireAtResult::None),
            WireAtResult::One(w) => (Some(w), WireAtResult::None),
            WireAtResult::Two([w1, w2]) => (Some(w1), WireAtResult::One(w2)),
        };

        *self = state;
        result
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct WireRangeMap1D {
    map: BTreeMap<Axis, NonZero<Axis>>
}
impl WireRangeMap1D {
    /// Creates a new wire range map.
    pub fn new() -> Self {
        Self::default()
    }

    /// If the index intersects a wire, then split the wires.
    /// Returns whether this split was successful.
    pub fn split(&mut self, index: Axis) -> Option<([Wire1D; 2], Wire1D)> {
        let WireAtResult::One(w) = self.wire_at(index) else {
            return None;
        };
        let left_len = NonZero::new(index - w.start)?;
        let right_len = NonZero::new(w.start + w.length.get() - index)?;

        let wire_len = self.map.get_mut(&w.start).unwrap();
        *wire_len = left_len;

        let add_result = self.map.insert(index, right_len);
        debug_assert!(add_result.is_none(), "Expected wire addition without conflict");
        Some(([Wire1D { start: w.start, length: left_len }, Wire1D { start: index, length: right_len }], w))
    }
    /// If the index is a joint between two wires, then merge the two wires.
    /// Returns three wires:
    /// - The two wires that are joined
    /// - The resulting joined wire
    pub fn join(&mut self, index: Axis) -> Option<([Wire1D; 2], Wire1D)> {
        let WireAtResult::Two([l, r]) = self.wire_at(index) else {
            return None;
        };

        let result = self.map.remove(&r.start);
        debug_assert_eq!(result, Some(r.length), "Expected successful wire removal");

        let wire_len = self.map.get_mut(&l.start).unwrap();
        *wire_len = wire_len.checked_add(r.length.get()).unwrap();

        Some(([l, r], Wire1D { start: l.start, length: *wire_len }))
    }
    /// Insert a wire.
    /// This returns which wires were actually added (if there was a wire already in the range).
    pub fn insert(&mut self, w: Wire1D) -> Vec<Wire1D> {
        let Wire1D { start, length } = w;
        let (mut a_start, a_end) = match self.wire_at(start) {
            // Start point doesn't intersect anything, so we tentatively start here:
            WireAtResult::None => (start, start + length.get()),

            // If there is a wire at this point, we need to remove the excess at the beginning.
            // We get the end of this wire (current_end),
            // and add up to the end of the inserting wire (adding_end).
            | WireAtResult::One(w)
            | WireAtResult::Two([_, w])
            => {
                let current_end = w.endpoints()[1];
                let adding_end = start + length.get();

                if current_end >= adding_end {
                    return vec![];
                }

                // current_end < adding_end
                (current_end, adding_end)
            },
        };
        let mut added = vec![];

        for w in self.map.range(a_start .. a_end)
            .map(|(&start, &length)| Wire1D { start, length })
        {
            let [c_start, c_end] = w.endpoints();

            // Add a wire if there's space between the addition space
            // and the current wires in the map
            if let Some(length) = NonZero::new(c_start - a_start) {
                added.push(Wire1D { start: a_start, length });
            }
            
            a_start = c_end;
        }
        if a_start < a_end && let Some(length) = NonZero::new(a_end - a_start) {
            added.push(Wire1D { start: a_start, length });
        }
        
        for w in &added {
            self.map.insert(w.start, w.length);
        }
        added
    }
    /// Removes a wire.
    /// This returns which wires were actually removed (if there was negative space in the range).
    pub fn remove(&mut self, w: Wire1D) -> Vec<Wire1D> {
        let Wire1D { start, length } = w;
        let end = start + length.get();
        self.split(start);
        self.split(end);

        let removed: Vec<_> = self.map.range(start .. start + length.get())
            .map(|(&start, &length)| Wire1D { start, length })
            .collect();
        
        for w in &removed {
            self.map.remove(&w.start);
        }
        removed
    }

    /// Gets the wires at the specified coordinate
    /// (including when the coordinate is not an endpoint, but intersects the wire).
    /// 
    /// The result is sorted by coordinate order.
    pub fn wire_at(&self, c: Axis) -> WireAtResult {
        let wire2 = self.map.get(&c)
            .map(|&length| Wire1D { start: c, length });
        
        let wire1 = self.map.range(..c)
            .next_back()
            .map(|(&start, &length)| Wire1D { start, length })
            .filter(|w| w.contains(c));
        
        match (wire1, wire2) {
            (None, None) => WireAtResult::None,
            (None, Some(w)) => WireAtResult::One(w),
            (Some(w), None) => WireAtResult::One(w),
            (Some(w1), Some(w2)) => WireAtResult::Two([w1, w2]),
        }
    }
    /// All wires of the range map.
    pub fn wires(&self) -> impl DoubleEndedIterator<Item=Wire1D> {
        self.map.iter()
            .map(|(&start, &length)| Wire1D { start, length })
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Wire1D {
    start: Axis,
    length: NonZero<Axis>
}
impl Wire1D {
    pub(super) fn new(start: Axis, length: NonZero<Axis>) -> Self {
        let _ = start.strict_add(length.get());
        Self { start, length }
    }

    pub fn to_2d(self, horizontal: bool, cross: Axis) -> Wire {
        match horizontal {
            true  => Wire::new_raw(self.start, cross, self.length, horizontal),
            false => Wire::new_raw(cross, self.start, self.length, horizontal),
        }
    }

    pub fn from_2d(w: Wire) -> (Self, Axis, bool) {
        let Wire { x, y, length, horizontal } = w;
        let (w1d, cross) = match horizontal {
            true  => (Self { start: x, length }, y),
            false => (Self { start: y, length }, x),
        };

        (w1d, cross, horizontal)
    }

    fn endpoints(self) -> [Axis; 2] {
        [self.start, self.start + self.length.get()]
    }
    fn contains(&self, c: Axis) -> bool {
        self.start <= c && c <= self.start + self.length.get()
    }
}

/// Convert a 1D split-join return value into a 2D split-join return value.
pub(super) fn sj_to_2d(split_join: ([Wire1D; 2], Wire1D), horizontal: bool, cross: Axis) -> ([Wire; 2], Wire) {
    let (operands, result) = split_join;
    (
        operands.map(|w| w.to_2d(horizontal, cross)),
        result.to_2d(horizontal, cross)
    )
}