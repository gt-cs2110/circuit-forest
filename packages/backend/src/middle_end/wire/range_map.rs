use std::collections::{BTreeMap, HashMap};
use std::num::NonZero;

use crate::middle_end::{Axis, Coord};
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
    pub fn remove(&mut self, w: Wire1D) -> (Vec<Wire1D>, Vec<Wire1D>) {
        let Wire1D { start, length } = w;
        let end = start + length.get();

        let mut removed = vec![];
        let mut added = vec![];

        // Split start joint
        if let Some(([spl, spr], joined)) = self.split(start) {
            removed.push(joined);
            
            added.push(spl);
            if spr.endpoints()[1] <= end {
                // spr is a synthetically added wire,
                // so if it exists and is not going to be split later,
                // then remove it so it doesn't get tracked later.
                self.map.remove(&spr.start);
            } else {
                added.push(spr);
            }
        }
        // Split end joint
        if let Some(([spl, spr], joined)) = self.split(end) {
            // joined also needs to be removed as long as
            // it wasn't synthetically created (in the last split step)
            if added.pop_if(|&mut w| w == joined).is_none() {
                removed.push(joined);
            }
            
            self.map.remove(&spl.start);
            added.push(spr);
        }

        // Add any extra wires between
        let len = removed.len();
        removed.extend({
            self.map.range(start .. start + length.get())
                .map(|(&start, &length)| Wire1D { start, length })
        });
        // Update map
        for w in &removed[len..] {
            self.map.remove(&w.start);
        }

        (removed, added)
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

// Two-dimension range map:

/// Convert a 1D split-join return value into a 2D split-join return value.
fn sj_to_2d(split_join: ([Wire1D; 2], Wire1D), horizontal: bool, cross: Axis) -> ([Wire; 2], Wire) {
    let (operands, result) = split_join;
    (
        operands.map(|w| w.to_2d(horizontal, cross)),
        result.to_2d(horizontal, cross)
    )
}

struct WireAtPointIter {
    entry: WireAtResult,
    horizontal: bool,
    cross: Axis,
}
impl WireAtPointIter {
    fn new(map: &WR1DSet, horizontal: bool, coord: Coord) -> Self {
        let (x, y) = coord;
        let (main, cross) = match horizontal {
            true  => (x, y),
            false => (y, x)
        };

        let entry = map.get(&cross).map_or_else(Default::default, |m| m.wire_at(main));
        Self { entry, horizontal, cross }
    }
}
impl Iterator for WireAtPointIter {
    type Item = Wire;

    fn next(&mut self) -> Option<Self::Item> {
        self.entry.next()
            .map(|w| w.to_2d(self.horizontal, self.cross))
    }
}
fn wires_all_iter(map: &WR1DSet, horizontal: bool) -> impl Iterator<Item=Wire> {
    map.iter().flat_map(move |(&cross, m)| m.wires().map(move |w| w.to_2d(horizontal, cross)))
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
    /// This returns the wires that are created as a result.
    pub fn add_wire(&mut self, w: Wire) -> Vec<Wire> {
        let (w1d, cross, horizontal) = Wire1D::from_2d(w);
        
        self.axis_map_mut(horizontal)
            .entry(cross)
            .or_default()
            .insert(w1d)
            .into_iter()
            .map(|w| w.to_2d(horizontal, cross))
            .collect()
    }

    /// Removes a wire from the range map.
    /// 
    /// This returns the wires that are deleted & added as a result of this removal.
    /// (A wire can be added if the wire overlaps another wire, requiring it to split).
    pub fn remove_wire(&mut self, w: Wire) -> (Vec<Wire>, Vec<Wire>) {
        use std::collections::hash_map::Entry;

        let (w1d, cross, horizontal) = Wire1D::from_2d(w);
        let Entry::Occupied(mut map1d) = self.axis_map_mut(horizontal).entry(cross) else {
            return (vec![], vec![]);
        };

        let (removed, added) = map1d.get_mut().remove(w1d);
        // Clear out excessive rows:
        if map1d.get().is_empty() {
            map1d.remove();
        }

        [removed, added]
            .map(|wires| wires.into_iter()
                .map(|w| w.to_2d(horizontal, cross))
                .collect()
            )
            .into()
    }
    /// Splits wire of the specified orientation at a given coordinate.
    pub fn split_wire(&mut self, horizontal: bool, c: Coord) -> Option<([Wire; 2], Wire)> {
        let (x, y) = c;
        let (main, cross) = match horizontal {
            true  => (x, y),
            false => (y, x)
        };

        self.axis_map_mut(horizontal)
            .get_mut(&cross)?
            .split(main) // Try to split at the coordinate in 1D
            .map(|sj| sj_to_2d(sj, horizontal, cross))
    }
    /// Tries to join two wires on a joint.
    pub fn join_wire(&mut self, c: Coord) -> Option<([Wire; 2], Wire)> {
        let (x, y) = c;
        let h_wire_at = self.horiz_wires.get_mut(&y)
            .map(|m| (m.wire_at(x), m));
        let v_wire_at = self.vert_wires.get_mut(&x)
            .map(|m| (m.wire_at(y), m));
        
        match (h_wire_at, v_wire_at) {
            (Some((WireAtResult::Two(_), h_map)), None | Some((WireAtResult::None, _))) 
                => Some(sj_to_2d(h_map.join(x).expect("successful join"), true, y)),
            (None | Some((WireAtResult::None, _)), Some((WireAtResult::Two(_), v_map))) 
                => Some(sj_to_2d(v_map.join(y).expect("successful join"), false, x)),
            _ => None
        }
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
            (None, Some(l)) => assert!(vw.entry(px).or_default().insert(Wire1D::new(py, l)).len() == 1, "there should not be two edges with the same starting endpoint in expected"),
            (Some(l), None) => assert!(hw.entry(py).or_default().insert(Wire1D::new(px, l)).len() == 1, "there should not be two edges with the same starting endpoint in expected"),
            (_, _) => panic!("all edges in expected should be horizontal or vertical")
        }
    }

    assert_eq!(actual.horiz_wires, hw, "expected horizontal wires to match");
    assert_eq!(actual.vert_wires, vw, "expected vertical wires to match");
}