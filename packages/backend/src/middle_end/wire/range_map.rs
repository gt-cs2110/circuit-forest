use std::collections::{BTreeMap, HashMap};
use std::num::NonZero;

use crate::middle_end::{Axis, Coord};
use crate::middle_end::wire::Wire;

/// Converts a coordinate of (x, y) into (main, cross)
/// (based on the `horizontal` value) or vice versa.
/// 
/// In other words, it switches the coordinate system
/// between XY and main-cross.
/// 
/// The "main" axis refers to the direction that the wire grows,
/// whereas the "cross" axis refers to the direction perpendicular.
fn switch_sys(coord: Coord, horizontal: bool) -> Coord {
    let (x, y) = coord;
    match horizontal {
        true  => (x, y),
        false => (y, x)
    }
}

/// A 1-dimensional wire.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
struct Wire1D {
    start: Axis,
    length: NonZero<Axis>
}
impl Wire1D {
    pub fn new(start: Axis, length: NonZero<Axis>) -> Self {
        let _ = start.strict_add(length.get());
        Self { start, length }
    }

    /// Projects the wire to 2D by specifying the wire's horizontality
    /// and the coordinate of the perpendicular axis 
    /// (the "cross" axis;
    ///     the x-coord if vertical, y-coord if horizontal).
    pub fn to_2d(self, cross: Axis, horizontal: bool) -> Wire {
        let (x, y) = switch_sys((self.start, cross), horizontal);
        Wire::new_raw(x, y, self.length, horizontal)
    }

    /// Converts a 2D wire to its 1D wire, the cross-axis coordinate,
    /// and the wire's horizontality.
    pub fn from_2d(w: Wire) -> (Self, Axis, bool) {
        let Wire { x, y, length, horizontal } = w;
        let (start, cross) = switch_sys((x, y), horizontal);
        (Self { start, length }, cross, horizontal)
    }

    fn endpoints(self) -> [Axis; 2] {
        [self.start, self.start + self.length.get()]
    }
    fn contains(&self, c: Axis) -> bool {
        self.start <= c && c <= self.start + self.length.get()
    }
}

/// The possible wires at a given point.
#[derive(PartialEq, Eq, Clone, Copy, Default)]
enum WireAtResult {
    /// No wires at this point.
    #[default]
    None,

    /// 1 wire adjacent left at this point
    /// (this point acts as the right endpoint of this wire).
    LWire(Wire1D),
    /// 1 wire at this point, which the point intersects with.
    Intersect(Wire1D),
    /// 1 wire adjacent right at this point
    /// (this point acts as the left endpoint of this wire).
    RWire(Wire1D),
    
    /// 2 wires at this point
    /// (this point is the endpoint of both).
    BiWire([Wire1D; 2])
}
impl Iterator for WireAtResult {
    type Item = Wire1D;

    fn next(&mut self) -> Option<Self::Item> {
        let (result, state) = match *self {
            // Zero wire:
            WireAtResult::None => (None, WireAtResult::None),
            // One wire:
            | WireAtResult::LWire(w)
            | WireAtResult::Intersect(w)
            | WireAtResult::RWire(w)
            => (Some(w), WireAtResult::None),
            // Two wires:
            WireAtResult::BiWire([w1, w2]) => (Some(w1), WireAtResult::RWire(w2)),
        };

        *self = state;
        result
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = match self {
            WireAtResult::None => 0,
            WireAtResult::LWire(_) => 1,
            WireAtResult::Intersect(_) => 1,
            WireAtResult::RWire(_) => 1,
            WireAtResult::BiWire(_) => 2,
        };

        (len, Some(len))
    }
}
impl ExactSizeIterator for WireAtResult {}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
struct WireRangeMap1D {
    map: BTreeMap<Axis, NonZero<Axis>>
}
impl WireRangeMap1D {
    /// Number of wires in this range map.
    #[expect(unused)]
    pub fn len(&self) -> usize {
        self.map.len()
    }
    /// Returns `true` if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Gets the wires at the specified coordinate
    /// (including when the coordinate is not an endpoint, but intersects the wire).
    /// 
    /// The result is sorted by coordinate order.
    #[must_use]
    pub fn wire_at(&self, c: Axis) -> WireAtResult {
        // Find wire that is directly adjacent right of `c`
        // (i.e., c is the left endpoint).
        let wire_right = self.map.get(&c)
            .map(|&length| Wire1D { start: c, length });
        
        // Find the wire that is directly adjacent left of `c`
        //    or intersecting the middle of the wire.
        let wire_left = self.map.range(..c)
            .next_back()
            .map(|(&start, &length)| Wire1D { start, length })
            .filter(|w| w.contains(c));
        
        match (wire_left, wire_right) {
            (None, None) => WireAtResult::None,
            (Some(w), None) if w.endpoints()[1] == c => WireAtResult::LWire(w),
            (Some(w), None) => WireAtResult::Intersect(w),
            (None, Some(w)) => WireAtResult::RWire(w),
            (Some(w1), Some(w2)) => WireAtResult::BiWire([w1, w2]),
        }
    }
    /// All wires of the range map.
    pub fn wires(&self) -> impl DoubleEndedIterator<Item=Wire1D> {
        self.map.iter()
            .map(|(&start, &length)| Wire1D { start, length })
    }

    /// Finds all wires which overlap with the specified 1D wire.
    pub fn overlapping_wires(&self, w: Wire1D) -> impl DoubleEndedIterator<Item=Wire1D> {
        let [wire_left, wire_right] = w.endpoints();

        let l = match self.wire_at(wire_left) {
            | WireAtResult::None
            | WireAtResult::LWire(_)
            | WireAtResult::RWire(_)
            | WireAtResult::BiWire(_) => wire_left,
            
            WireAtResult::Intersect(w) => w.start,
        };
        let r = match self.wire_at(wire_right) {
            | WireAtResult::None
            | WireAtResult::LWire(_)
            | WireAtResult::RWire(_)
            | WireAtResult::BiWire(_) => wire_right,

            WireAtResult::Intersect(w) => w.start + w.length.get(),
        };
        
        self.map.range(l..r)
            .map(|(&start, &length)| Wire1D::new(start, length))
    }


    /// Insert a wire.
    /// 
    /// This returns whether the wire was added.
    /// Note that for this wire to be added, it must not overlap any other wire.
    pub fn insert(&mut self, w: Wire1D) -> bool {
        let [start, end] = w.endpoints();

        let is_not_overlapping = self.map.range(..end).next_back()
            .is_none_or(|(st, sz)| st + sz.get() <= start);
        if is_not_overlapping {
            self.map.insert(w.start, w.length);
            true
        } else {
            false
        }
    }
    /// Removes a wire.
    /// 
    /// This returns whether the wire was removed.
    /// Note that the wire must exist in the map EXACTLY as specified by the argument
    /// in order for removal to succeed.
    pub fn remove(&mut self, w: Wire1D) -> bool {
        use std::collections::btree_map::Entry;

        if let Entry::Occupied(e) = self.map.entry(w.start) && e.get() == &w.length {
            e.remove();
            true
        } else {
            false
        }
    }

    /// If the index intersects a wire, then split the wires.
    /// 
    /// Returns the wire that was split.
    pub fn split(&mut self, index: Axis) -> Option<Wire1D> {
        let WireAtResult::Intersect(w) = self.wire_at(index) else {
            return None;
        };
        let left_len = NonZero::new(index - w.start)?;
        let right_len = NonZero::new(w.start + w.length.get() - index)?;

        let wire_len = self.map.get_mut(&w.start).unwrap();
        *wire_len = left_len;

        let add_result = self.map.insert(index, right_len);
        debug_assert!(add_result.is_none(), "Expected wire addition without conflict");
        
        Some(w)
    }
}

// Two-dimension range map:

struct WireAtPointIter {
    entry: WireAtResult,
    horizontal: bool,
    cross: Axis,
}
impl WireAtPointIter {
    fn new(map: &WR1DSet, horizontal: bool, coord: Coord) -> Self {
        let (main, cross) = switch_sys(coord, horizontal);
        let entry = map.get(&cross).map_or_else(Default::default, |m| m.wire_at(main));
        Self { entry, horizontal, cross }
    }
}
impl Iterator for WireAtPointIter {
    type Item = Wire;

    fn next(&mut self) -> Option<Self::Item> {
        self.entry.next()
            .map(|w| w.to_2d(self.cross, self.horizontal))
    }
}
fn wires_all_iter(map: &WR1DSet, horizontal: bool) -> impl Iterator<Item=Wire> {
    map.iter().flat_map(move |(&cross, m)| m.wires().map(move |w| w.to_2d(cross, horizontal)))
}
type WR1DSet = HashMap<Axis, WireRangeMap1D>;
/// A helper struct which is Coord-indexable, indicating whether a wire exists along a coord.
#[derive(Default)]
pub struct WireRangeMap {
    /// All horizontal wires. This is Map<y, Map<start x, length>>.
    horiz_wires: WR1DSet,

    /// All vertical wires. This is Map<x, Map<start y, length>>.
    vert_wires: WR1DSet
}
impl WireRangeMap {
    /// Adds a wire to the range map.
    /// 
    /// This returns whether adding the wire was successful
    /// (this wire cannot overlap any other wire).
    pub fn add_wire(&mut self, w: Wire) -> bool {
        let (w1d, cross, horizontal) = Wire1D::from_2d(w);
        
        self.axis_map_mut(horizontal)
            .entry(cross)
            .or_default()
            .insert(w1d)
    }

    /// Removes a wire from the range map.
    /// 
    /// This returns whether removing the wire was successful
    /// (this wire must exist exactly as specified in the map).
    pub fn remove_wire(&mut self, w: Wire) -> bool {
        use std::collections::hash_map::Entry;

        let (w1d, cross, horizontal) = Wire1D::from_2d(w);
        if let Entry::Occupied(mut map1d) = self.axis_map_mut(horizontal).entry(cross) {
            let removed = map1d.get_mut().remove(w1d);
            if removed && map1d.get().is_empty() {
                map1d.remove();
            }
            removed
        } else {
            false
        }
    }
    /// Splits wire of the specified orientation at a given coordinate.
    /// 
    /// This returns the wire that was split.
    pub fn split_wire(&mut self, horizontal: bool, c: Coord) -> Option<Wire> {
        let (main, cross) = switch_sys(c, horizontal);
        self.axis_map_mut(horizontal)
            .get_mut(&cross)?
            .split(main) // Try to split at the coordinate in 1D
            .map(|w| w.to_2d(cross, horizontal))
    }

    /// Gets all the wires which overlap with the specified wire, on the same axis.
    pub fn parallel_overlapping_wires(&self, w: Wire) -> impl DoubleEndedIterator<Item = Wire> {
        let (w1d, cross, horizontal) = Wire1D::from_2d(w);

        self.axis_map(horizontal)
            .get(&cross)
            .into_iter()
            .flat_map(move |m| m.overlapping_wires(w1d))
            .map(move |w| w.to_2d(cross, horizontal))
    }

    /// Gets the wire map for the corresponding `horizontal` value.
    fn axis_map(&self, horizontal: bool) -> &WR1DSet {
        match horizontal {
            true  => &self.horiz_wires,
            false => &self.vert_wires
        }
    }

    /// Gets the wire map for the corresponding `horizontal` value.
    fn axis_map_mut(&mut self, horizontal: bool) -> &mut WR1DSet {
        match horizontal {
            true  => &mut self.horiz_wires,
            false => &mut self.vert_wires
        }
    }

    fn wires_at_coord_dir(&self, horizontal: bool, c: Coord) -> WireAtPointIter {
        WireAtPointIter::new(self.axis_map(horizontal), horizontal, c)
    }
    /// Gets all of the wires at the coord
    /// (including those that coord only intersects, not necessarily just the ones coord is an endpoint of).
    pub fn wires_at_coord(&self, c: Coord) -> impl Iterator<Item=Wire> {
        self.wires_at_coord_dir(true, c)
            .chain(self.wires_at_coord_dir(false, c))
    }

    /// Gets all of the wires defined in the map.
    pub fn wires(&self) -> impl Iterator<Item=Wire> {
        wires_all_iter(&self.horiz_wires, true)
            .chain(wires_all_iter(&self.vert_wires, false))
    }
}
impl std::fmt::Debug for WireRangeMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct WR1DFmt<'a>(&'a WR1DSet, bool);
        impl std::fmt::Debug for WR1DFmt<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let &WR1DFmt(map, horizontal) = self;
                f.debug_set()
                    .entries(wires_all_iter(map, horizontal))
                    .finish()
            }
        }

        f.debug_struct("WireRangeMap")
            .field("horiz_wires", &WR1DFmt(&self.horiz_wires, true))
            .field("vert_wires", &WR1DFmt(&self.vert_wires, false))
            .finish()
    }
}

/// Asserts the range map matches an expected edge set.
#[cfg(test)]
pub(super) fn assert_range_map(actual: &WireRangeMap, edges: impl IntoIterator<Item=(Coord, Coord)>) {
    use crate::middle_end::wire::minmax;

    let mut hw = HashMap::<_, WireRangeMap1D>::new();
    let mut vw = HashMap::<_, WireRangeMap1D>::new();
    for (p, q) in edges {
        let [(px, py), (qx, qy)] = minmax(p, q);
        match (NonZero::new(qx - px), NonZero::new(qy - py)) {
            (None, None) => panic!("all edges in expected should be non-zero-length"),
            (None, Some(l)) => assert!(vw.entry(px).or_default().insert(Wire1D::new(py, l)), "addition of wire should've been successful"),
            (Some(l), None) => assert!(hw.entry(py).or_default().insert(Wire1D::new(px, l)), "addition of wire should've been successful"),
            (_, _) => panic!("all edges in expected should be horizontal or vertical")
        }
    }

    assert_eq!(actual.horiz_wires, hw, "expected horizontal wires to match");
    assert_eq!(actual.vert_wires, vw, "expected vertical wires to match");
}