mod range_map;

use std::collections::{HashMap, HashSet};
use std::num::NonZero;

use petgraph::Undirected;
use petgraph::prelude::GraphMap;
use petgraph::visit::{Bfs, Walker};

use crate::circuit::graph::ValueKey;
use crate::middle_end::wire::range_map::{Wire1D, WireAtResult, WireRangeMap1D, sj_to_2d};
use crate::middle_end::{Axis, Coord, UIKey};

/// A key to attach onto the wire set graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum MeshKey {
    /// A joint (a wire point)
    WireJoint(Coord),
    /// A tunnel.
    Tunnel(UIKey)
}
impl From<Coord> for MeshKey {
    fn from(value: Coord) -> Self {
        MeshKey::WireJoint(value)
    }
}

/// The result type for [`WireSet::add_wire`].
/// 
/// This enum indicates whether two [`ValueKey`]s need to be joined.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum AddWireResult {
    /// No joining is necessary.
    /// The [`ValueKey`] provided is the key of the added wire.
    NoJoin(ValueKey),

    /// Joining is necessary.
    /// The parameters are:
    /// - The coordinate to start a flood fill from.
    /// - The value key the new wire is set to.
    /// - The value keys which need to be replaced with the new wire's key.
    Join(Coord, ValueKey, Vec<ValueKey>)
}

/// The result type for [`WireSet::remove_wire`].
/// 
/// This enum indicates whether a [`ValueKey`] must be split into two.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum RemoveWireResult {
    /// No splitting is necessary.
    /// The [`ValueKey`] provided is the key of the two endpoints.
    NoSplit(ValueKey),
    /// Splitting is necessary.
    /// The parameters are:
    /// - The coordinate to start a flood fill from.
    /// - The [`ValueKey`] which needs to be split.
    /// - A list of coordinates which are on one half of the split ValueKey
    ///   (the same half as the coordinate parameter).
    Split(Coord, ValueKey, HashSet<Coord>)
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
struct WireRangeMap {
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
    /// This returns the wires that are deleted as a result.
    pub fn remove_wire(&mut self, w: Wire) -> Vec<Wire> {
        use std::collections::hash_map::Entry;

        let (w1d, cross, horizontal) = Wire1D::from_2d(w);
        let Entry::Occupied(mut map1d) = self.axis_map_mut(horizontal).entry(cross) else {
            return vec![];
        };

        let result = map1d.get_mut().remove(w1d);
        if map1d.get().is_empty() {
            map1d.remove();
        }

        result.into_iter()
            .map(|w| w.to_2d(horizontal, cross))
            .collect()
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

type WireGraph = GraphMap<MeshKey, ValueKey, Undirected>;
/// The connection of wires in a circuit.
#[derive(Debug, Default)]
pub struct WireSet {
    wires: WireGraph,
    ranges: WireRangeMap,
}
impl WireSet {
    /// Find the ValueKey corresponding to a coordinate.
    /// 
    /// This is None if the coordinate is not connected to a wire.
    pub fn find_key(&self, p: Coord) -> Option<ValueKey> {
        self.wires.edges(p.into())
            .next()
            .map(|(_, _, &k)| k)
    }

    /// Checks if there is a wire to split, and splitting it into two if needed.
    /// 
    /// This function accepts the coordinate to split at and the orientation of the wire
    /// the coordinate is an endpoint of.
    /// This tries to look for any wires which are perpendicular to this wire,
    /// splitting it if one is found.
    /// 
    /// Returns whether a split was successful.
    fn split_wire_on_joint(&mut self, c: Coord, horizontal: bool) {
        // Get perpendicular map
        let mut it = self.ranges.wires_at_coord_dir(!horizontal, c)
            .filter(|w| w.split(c).is_some());
        
        let Some(w) = it.next() else { return };
        debug_assert!(it.next().is_none(), "Expected only one splittable wire");

        // Split wires in graph:
        let [p, q] = w.endpoints();
        let Some(k) = self.wires.remove_edge(p.into(), q.into()) else {
            unreachable!("Expected wire to split to exist");
        };
        self.wires.add_edge(p.into(), c.into(), k);
        self.wires.add_edge(c.into(), q.into(), k);
        // Split wires in map:
        assert!(
            self.ranges.split_wire(w.horizontal, c).is_some(),
            "Expected wire to be splittable"
        );
    }

    /// Removes node if it is not connected to any wire.
    fn remove_if_singleton(&mut self, n: Coord) {
        let n = n.into();
        if self.wires.neighbors(n).next().is_none() {
            self.wires.remove_node(n);
        }
    }
    /// Add a wire to the graph, connecting points p and q.
    /// A `new_vk` callback needs to be provided in case edge pq is disconnected 
    /// from the rest of the graph and needs a new key.
    /// 
    /// This function may add additional wires (e.g., if a connection would result in an intersection)
    /// or subsume wires which already exist (e.g., to extend a wire).
    /// 
    /// If this function returns None, the wire could not be added.
    /// Otherwise, this function returns data needed to merge two groups of wires
    /// with different [`ValueKey`]s (if applicable).
    #[must_use]
    pub fn add_wire(&mut self, p: Coord, q: Coord, new_vk: impl FnOnce() -> ValueKey) -> Option<AddWireResult> {
        let w = Wire::from_endpoints(p, q)?;

        // If horizontal or vertical, these two points can be connected.
        let [p, q] = w.endpoints();
        let pk = self.find_key(p);
        let qk = self.find_key(q);

        // If endpoints intersect the middle of a wire, create an intersection:
        self.split_wire_on_joint(p, w.horizontal);
        self.split_wire_on_joint(q, w.horizontal);
        // All keys along wire:
        let keys: HashSet<_> = w.coord_iter()
            .filter_map(|c| self.find_key(c))
            .collect();

        // Add to wire maps:
        let mut deleted = vec![];
        let mut added = vec![];
        let mut update_set = |deleted_wires: &[Wire], merged_wire| {
            added.pop_if(|w| deleted_wires.contains(w));
            deleted.extend_from_slice(deleted_wires);
            added.push(merged_wire);
        };
        for subwire in self.ranges.add_wire(w) {
            // For the two endpoints, try merging wires,
            // and keeping track of which wires are added/deleted
            let [l, r] = subwire.endpoints();
            let jl = self.ranges.join_wire(l);
            let jr = self.ranges.join_wire(r);
            match (jl, jr) {
                (Some((l_deleted_wires, l_merged)), Some((r_deleted_wires, r_merged))) => {
                    update_set(&l_deleted_wires, l_merged);
                    update_set(&r_deleted_wires, r_merged);
                },
                (Some((deleted_wires, merged)), None) | (None, Some((deleted_wires, merged))) => {
                    update_set(&deleted_wires, merged);
                },
                (None, None) => update_set(&[], subwire)
            }
        }
        if deleted.is_empty() && added.is_empty() {
            return None;
        }

        // Delete each wire in `deleted`.
        for w in deleted {
            let [w0, w1] = w.endpoints();
            let removed = self.wires.remove_edge(w0.into(), w1.into());
            debug_assert!(
                removed.is_none_or(|k| keys.contains(&k)),
                "Removal of edge should have value key which is already accounted for"
            );

            self.remove_if_singleton(w0);
            self.remove_if_singleton(w1);
        }

        // Determine which key we should use to fill & whether a join is needed
        let keys = Vec::from_iter(keys);
        let (fill_key, result) = match keys.as_slice() {
            [] => {
                let new_key = new_vk();
                (new_key, AddWireResult::NoJoin(new_key))
            },
            &[k] => (k, AddWireResult::NoJoin(k)),
            &[k1, k2, ..] => {
                // If multiple keys, then we know we need to eventually join
                // and we need to determine which key to use.
                // 
                // fill_point: Where a flood fill needs to start
                // fill_key: The key to temporarily fill the wire with
                // post_fill_key: The key to actually fill with (using flood fill)
                //
                // This is set up this way so that any adjacent wires to this wire
                // are all flood-filled properly.
                let (fill_point, post_fill_key, fill_key) = match (pk, qk) {
                    (Some(pk), _) => (p, pk, *keys.iter().find(|&&k| k != pk).unwrap()),
                    (_, Some(qk)) => (q, qk, *keys.iter().find(|&&k| k != qk).unwrap()),
                    _ => (p, k1, k2)
                };

                (fill_key, AddWireResult::Join(fill_point, post_fill_key, keys))
            }
        };
        // Add all the wires
        self.wires.extend(
            added.into_iter().map(|w| {
                let [l, r] = w.endpoints();
                (l.into(), r.into(), fill_key)
            })
        );
        // Break up any new wires with any joints that connect to this wire.
        for c in w.coord_iter() {
            let intersecting = self.wires.neighbors(c.into())
                .any(|other| match other {
                    MeshKey::WireJoint(r) => Wire::from_endpoints(c, r)
                        .is_some_and(|w2| w.horizontal != w2.horizontal),
                    MeshKey::Tunnel(_) => false,
                });
            if intersecting {
                self.split_wire_on_joint(c, !w.horizontal);
            }
        }

        Some(result)
    }
    
    /// Removes the wire from the graph between p and q.
    /// 
    /// Note that this function only removes wires that are directly connected by joints
    /// in the circuit.
    /// 
    /// If this function returns None, the wire does not exist & could not be removed.
    /// Otherwise, this function returns data needed to split a [`ValueKey`] (if applicable).
    #[must_use]
    pub fn remove_wire(&mut self, p: Coord, q: Coord) -> Option<RemoveWireResult> {
        let [p, q] = minmax(p, q);
        
        // Remove from wire graph:
        let e = self.wires.remove_edge(p.into(), q.into())?;

        // Remove from wire map:
        let Some(w) = Wire::from_endpoints(p, q) else {
            unreachable!("({p:?}, {q:?}) must constitute a valid wire due to being successfully removed from the graph");
        };
        self.ranges.remove_wire(w);

        // If removed, also check if a ValueKey needs to be split
        let joints: HashSet<_> = Bfs::new(&self.wires, q.into())
            .iter(&self.wires)
            .filter_map(|m| match m {
                MeshKey::WireJoint(c) => Some(c),
                MeshKey::Tunnel(_) => None,
            })
            .collect();
        
        // Remove extraneous, disconnected nodes
        self.remove_if_singleton(p);
        self.remove_if_singleton(q);

        let result = match joints.contains(&p) {
            true  => RemoveWireResult::NoSplit(e),
            false => RemoveWireResult::Split(q, e, joints),
        };
        Some(result)
    }

    /// Replaces the [`ValueKey`] of all the wires connecting to the specified Coord
    /// with the specified [`ValueKey`].
    /// 
    /// This works as long as Coord only has 1 [`ValueKey`] 
    /// or is connected to 2 [`ValueKey`]s (one of which is the flood key).
    pub(crate) fn flood_fill(&mut self, p: Coord, flood_key: ValueKey) {
        let mut frontier = vec![MeshKey::WireJoint(p)];

        while let Some(k) = frontier.pop() {
            let edges_to_flood: Vec<_> = self.wires.edges(k)
                .filter(|&(_, _, &key)| key != flood_key)
                .map(|(n1, n2, _)| (n1, n2))
                .collect();
            for (n1, n2) in edges_to_flood {
                if let Some(k) = self.wires.edge_weight_mut(n1, n2) {
                    *k = flood_key;
                }
                frontier.push(n2);
            }
        }
    }

    /// Gets all wire segments coinciding at the specified coords.
    /// 
    /// This returns all wire segments, including segments that this coord
    /// is in the middle of.
    pub fn wires_at_coord(&self, c: Coord) -> impl Iterator<Item = Wire> {
        self.ranges.wires_at_coord(c)
    }
}

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

    /// Splits a wire into two, returning None if the specified coordinate does not intersect the wire.
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

    /// Joins two wires, returning None if the two wires would not join into a 1D wire.
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use slotmap::SlotMap;

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

    #[test]
    fn wire_split() {
        // Horizontal
        let [p, m, q] = [(1, 2), (1, 9), (1, 11)];
        let w = Wire::from_endpoints(p, q).unwrap();
        let split = [
            Wire::from_endpoints(p, m).unwrap(),
            Wire::from_endpoints(m, q).unwrap(),
        ];
        assert_eq!(w.split(m), Some(split));
        assert_eq!(w.split((2, 9)), None);
        
        // Vertical
        let [p, m, q] = [(2, 6), (9, 6), (11, 6)];
        let w = Wire::from_endpoints(p, q).unwrap();
        let split = [
            Wire::from_endpoints(p, m).unwrap(),
            Wire::from_endpoints(m, q).unwrap(),
        ];
        assert_eq!(w.split(m), Some(split));
        assert_eq!(w.split((9, 7)), None);
    }

    #[test]
    fn wire_join() {
        // Horizontal
        let [p, m, q] = [(1, 2), (1, 9), (1, 11)];
        let w1 = Wire::from_endpoints(p, m).unwrap();
        let w2 = Wire::from_endpoints(m, q).unwrap();
        let w = Wire::from_endpoints(p, q).unwrap();
        assert_eq!(w1.join(w2), Some(w));
                
        // Vertical
        let [p, m, q] = [(2, 6), (9, 6), (11, 6)];
        let w1 = Wire::from_endpoints(p, m).unwrap();
        let w2 = Wire::from_endpoints(m, q).unwrap();
        let w = Wire::from_endpoints(p, q).unwrap();
        assert_eq!(w1.join(w2), Some(w));

        // None cases
        let [p1, q1] = [(1, 2), (1, 9)];
        let [p2, q2] = [(2, 9), (2, 11)];
        let w1 = Wire::from_endpoints(p1, q1).unwrap();
        let w2 = Wire::from_endpoints(p2, q2).unwrap();
        assert_eq!(w1.join(w2), None);

        let [p1, q1] = [(2, 1), (9, 1)];
        let [p2, q2] = [(9, 2), (11, 2)];
        let w1 = Wire::from_endpoints(p1, q1).unwrap();
        let w2 = Wire::from_endpoints(p2, q2).unwrap();
        assert_eq!(w1.join(w2), None);
    }

    fn keygen() -> impl FnMut() -> ValueKey {
        let mut map = SlotMap::with_key();
        move || map.insert(())
    }
    /// Asserts nodes of the graph are exactly the specified node list.
    fn assert_graph_nodes<const N: usize>(graph: &WireGraph, nodes: [Coord; N]) {
        let actual: BTreeSet<_> = graph.nodes().collect();
        let expected: BTreeSet<_> = nodes.into_iter().map(MeshKey::WireJoint).collect();
        assert_eq!(actual, expected, "nodes in graph should match");
    }
    fn assert_graph_edges<const N: usize>(graph: &WireGraph, all_edges: [(ValueKey, Vec<(Coord, Coord)>); N]) {
        let expected_edgemap = HashMap::from(all_edges);
        
        let mut edgelist: Vec<_> = graph.all_edges().collect();
        edgelist.sort_by_key(|&(_, _, &vk)| vk);
        // Check each chunk contains the same keys:
        for chunk in edgelist.chunk_by(|(_, _, vk0), (_, _, vk1)| vk0 == vk1) {
            let key = chunk[0].2;

            let mut actual_edges: Vec<_> = chunk.iter()
                .map(|&(a, b, _)| minmax(a, b))
                .collect();
            let mut expected_edges: Vec<_> = expected_edgemap[key]
                .iter()
                .map(|&(l, r)| minmax(MeshKey::from(l), MeshKey::from(r)))
                .collect();

            actual_edges.sort();
            expected_edges.sort();
            assert_eq!(actual_edges, expected_edges, "edges for key {key:?} should match")
        }
    }
    fn assert_range_map(actual: &WireRangeMap, edges: impl IntoIterator<Item=(Coord, Coord)>) {
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

    /// Assert edges of the graph are exactly the specified edge list.
    #[test]
    fn wireset_add_basic() {
        let mut keygen = keygen();
        let mut ws = WireSet::default();

        let nodes @ [n00, n01, n11, n12, n02] = [(0, 0), (0, 4), (4, 4), (4, 10), (0, 10)];

        // Add nodes:
        let Some(AddWireResult::NoJoin(key)) = ws.add_wire(n00, n01, &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins")
        };
        assert_eq!(ws.add_wire(n01, n11, &mut keygen), Some(AddWireResult::NoJoin(key)));
        assert_eq!(ws.add_wire(n11, n12, &mut keygen), Some(AddWireResult::NoJoin(key)));
        assert_eq!(ws.add_wire(n01, n02, &mut keygen), Some(AddWireResult::NoJoin(key)));

        // Check wire set was constructed correctly
        assert_graph_nodes(&ws.wires, nodes);

        let edges = [(n00, n01), (n01, n11), (n11, n12), (n01, n02)];
        assert_graph_edges(&ws.wires, [(key, edges.to_vec())]);
        assert_range_map(&ws.ranges, edges);
    }

    #[test]
    fn wireset_add_fail() {
        let mut keygen = keygen();
        let mut ws = WireSet::default();

        assert!(matches!(ws.add_wire((0, 0), (0, 1), &mut keygen), Some(AddWireResult::NoJoin(_))));
        assert!(ws.add_wire((0, 0), (0, 0), &mut keygen).is_none()); // zero wire
        assert!(ws.add_wire((0, 0), (0, 1), &mut keygen).is_none()); // same wire
        assert!(ws.add_wire((0, 0), (1, 2), &mut keygen).is_none()); // diagonal wire
    }

    #[test]
    fn wireset_add_join() {
        let mut keygen = keygen();
        let mut ws = WireSet::default();

        let nodes @ [n00, n01, n02, n10, n11, n12] = [
            (2, 2), (2, 3), (1, 3),
            (3, 4), (3, 3), (4, 3),
        ];

        // Add nodes ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let Some(AddWireResult::NoJoin(k0)) = ws.add_wire(n00, n01, &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins");
        };
        assert_eq!(ws.add_wire(n01, n02, &mut keygen), Some(AddWireResult::NoJoin(k0)));
        
        let Some(AddWireResult::NoJoin(k1)) = ws.add_wire(n10, n11, &mut keygen) else {
            panic!("Expected second wire add to be successful and require no joins");
        };
        assert_eq!(ws.add_wire(n11, n12, &mut keygen), Some(AddWireResult::NoJoin(k1)));
        
        // Check wire set was constructed correctly
        assert_graph_nodes(&ws.wires, nodes);

        let edges = [
            (n00, n01), (n01, n02),
            (n10, n11), (n11, n12)
        ];
        assert_graph_edges(&ws.wires, [
            (k0, edges[0..2].to_vec()),
            (k1, edges[2..4].to_vec()),
        ]);
        assert_range_map(&ws.ranges, edges);

        // Join ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let Some(AddWireResult::Join(ffpt, src_key, dst_key)) = ws.add_wire(n01, n11, &mut keygen) else {
            panic!("Expected join")
        };
        assert!(ffpt == n01 || ffpt == n11);
        assert!(src_key == k0 || src_key == k1);
        assert!(dst_key.contains(&k0) && dst_key.contains(&k1));

        // Check wire set was constructed correctly
        assert_graph_nodes(&ws.wires, nodes);

        assert_eq!(ws.wires.edge_count(), 5);
        assert_range_map(&ws.ranges, [
            (n00, n01), (n01, n02),
            (n10, n11), (n11, n12),
            (n01, n11)
        ]);
    }

    #[test]
    fn wireset_add_extend_wire() {
        // Test that if wire of same orientation is attached to the end,
        // it results in a proper extension of the wire (instead of the creation of a new wire)

        let mut keygen = keygen();
        let mut ws = WireSet::default();

        let [n00, n01, n02, n03, n04, n05, n13] = [
            (1, 1), (3, 1), (5, 1), (7, 1), (9, 1), (11, 1),
            (7, 3),
        ];

        // Add nodes (1) ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let Some(AddWireResult::NoJoin(key)) = ws.add_wire(n01, n02, &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins");
        };
        assert_eq!(ws.add_wire(n02, n03, &mut keygen), Some(AddWireResult::NoJoin(key)));
        assert_eq!(ws.add_wire(n00, n01, &mut keygen), Some(AddWireResult::NoJoin(key)));
        
        // Check wire set was constructed correctly
        assert_graph_nodes(&ws.wires, [n00, n03]);
        
        assert_graph_edges(&ws.wires, [
            (key, vec![(n00, n03)]),
        ]);
        assert_range_map(&ws.ranges, [(n00, n03)]);

        // Add nodes (2) ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        assert_eq!(ws.add_wire(n13, n03, &mut keygen), Some(AddWireResult::NoJoin(key)));
        assert_eq!(ws.add_wire(n03, n04, &mut keygen), Some(AddWireResult::NoJoin(key)));
        assert_eq!(ws.add_wire(n04, n05, &mut keygen), Some(AddWireResult::NoJoin(key)));

        // Check wire set was constructed correctly
        assert_graph_nodes(&ws.wires, [n00, n03, n13, n05]);
        
        let edges = [(n00, n03), (n03, n13), (n03, n05)];
        assert_graph_edges(&ws.wires, [(key, edges.to_vec())]);
        assert_range_map(&ws.ranges, edges);
    }

    #[test]
    fn wireset_add_subset() {
        let mut keygen = keygen();
        let mut ws = WireSet::default();

        let [n00, n01, n02, n03] = [
            (1, 1), (3, 1), (5, 1), (7, 1)
        ];

        let Some(AddWireResult::NoJoin(key)) = ws.add_wire(n00, n02, &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins");
        };
        assert_eq!(ws.add_wire(n01, n03, &mut keygen), Some(AddWireResult::NoJoin(key)));
        // Nothing added:
        assert!(ws.add_wire(n00, n03, &mut keygen).is_none());
        assert!(ws.add_wire(n00, n02, &mut keygen).is_none());
        
        // Check wire set was constructed correctly
        assert_graph_nodes(&ws.wires, [n00, n03]);

        assert_graph_edges(&ws.wires, [
            (key, vec![(n00, n03)]),
        ]);
        assert_range_map(&ws.ranges, [(n00, n03)]);
    }

    #[test]
    fn wireset_add_split_wire() {
        // Test that adding a wire that connects 
        // to the middle of another wire creates a junction.
        let mut keygen = keygen();
        let mut ws = WireSet::default();

        let [n00, n01, n02, n11] = [
            (1, 1), (3, 1), (5, 1), (3, 3),
        ];

        // Add nodes (1) ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let Some(AddWireResult::NoJoin(key)) = ws.add_wire(n00, n02, &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins");
        };
        assert_eq!(ws.add_wire(n01, n11, &mut keygen), Some(AddWireResult::NoJoin(key)));

        // Check wire set was constructed correctly
        assert_graph_nodes(&ws.wires, [n00, n01, n02, n11]);

        let edges = [(n00, n01), (n01, n02), (n01, n11)];
        assert_graph_edges(&ws.wires, [(key, edges.to_vec())]);
        assert_range_map(&ws.ranges, edges);
    }

    #[test]
    fn wireset_add_draw_along() {
        // Test that if a wire's endpoint is in the middle of
        // a newly created wire,
        // the wire is automatically split with a junction.
        let mut keygen = keygen();
        let mut ws = WireSet::default();

        let [n00, n01, n02, n11] = [
            (1, 1), (3, 1), (5, 1), (3, 3),
        ];

        // Add nodes (1) ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let Some(AddWireResult::NoJoin(key)) = ws.add_wire(n01, n11, &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins");
        };
        assert_eq!(ws.add_wire(n00, n02, &mut keygen), Some(AddWireResult::NoJoin(key)));

        // Check wire set was constructed correctly
        assert_graph_nodes(&ws.wires, [n00, n01, n02, n11]);

        let edges = [(n00, n01), (n01, n02), (n01, n11)];
        assert_graph_edges(&ws.wires, [(key, edges.to_vec())]);
        assert_range_map(&ws.ranges, edges);
    }

    fn assert_split(
        result: Option<RemoveWireResult>, key: ValueKey,
        left_joint: Coord, left_ends: impl IntoIterator<Item=Coord>,
        right_joint: Coord, right_ends: impl IntoIterator<Item=Coord>,
    ) {
        let Some(result) = result else {
            panic!("Expected removal to succeed");
        };
        let RemoveWireResult::Split(p, skey, set) = result else {
            panic!("Expected removal of wire to induce split");
        };

        assert_eq!(skey, key, "Expected correct key to be split");
        assert!(
            (p, &set) == (left_joint, &HashSet::from_iter(left_ends)) ||
            (p, &set) == (right_joint, &HashSet::from_iter(right_ends)),
            "Expected split to properly match either left or right side of wire"
        )
    }
    #[test]
    fn wireset_remove_basic() {
        let mut keygen = keygen();
        let mut ws = WireSet::default();

        let [n00, n01, n11, n12, n02] = [(0, 0), (0, 4), (4, 4), (4, 10), (0, 10)];

        // Add nodes:
        let Some(AddWireResult::NoJoin(key)) = ws.add_wire(n00, n01, &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins")
        };
        assert_eq!(ws.add_wire(n01, n11, &mut keygen), Some(AddWireResult::NoJoin(key)));
        assert_eq!(ws.add_wire(n11, n12, &mut keygen), Some(AddWireResult::NoJoin(key)));
        assert_eq!(ws.add_wire(n01, n02, &mut keygen), Some(AddWireResult::NoJoin(key)));

        // Remove nodes:
        assert_split(
            ws.remove_wire(n01, n02), key,
            n01, [n00, n01, n11, n12],
            n02, [n02]
        );
        assert_split(
            ws.remove_wire(n11, n12), key,
            n11, [n00, n01, n11],
            n12, [n12]
        );
        assert_split(
            ws.remove_wire(n01, n11), key,
            n01, [n00, n01],
            n11, [n11]
        );
        assert_split(
            ws.remove_wire(n00, n01), key,
            n00, [n00],
            n01, [n01]
        );

        // Check corre0ct construction
        assert_graph_nodes(&ws.wires, []);
        assert_graph_edges(&ws.wires, []);
        assert_range_map(&ws.ranges, []);
    }

    #[test]
    fn wireset_remove_fail() {
        let mut keygen = keygen();
        let mut ws = WireSet::default();

        assert_eq!(ws.remove_wire((0, 0), (0, 1)), None); // Empty

        let Some(AddWireResult::NoJoin(_)) = ws.add_wire((0, 1), (0, 2), &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins")
        };
        assert_eq!(ws.remove_wire((0, 1), (0, 3)), None); // Too large
        assert_eq!(ws.remove_wire((0, 5), (0, 9)), None); // Does not exist
        assert_eq!(ws.remove_wire((0, 1), (3, 3)), None); // Diagonal
    }

    #[test]
    fn wireset_remove_split() {
        let mut keygen = keygen();
        let mut ws = WireSet::default();

        let nodes @ [n00, n01, n02, n10, n11, n12] = [
            (2, 2), (2, 3), (1, 3),
            (3, 4), (3, 3), (4, 3),
        ];

        // Add nodes ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let Some(AddWireResult::NoJoin(k0)) = ws.add_wire(n00, n01, &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins");
        };
        assert_eq!(ws.add_wire(n01, n02, &mut keygen), Some(AddWireResult::NoJoin(k0)));
        assert_eq!(ws.add_wire(n01, n11, &mut keygen), Some(AddWireResult::NoJoin(k0)));
        assert_eq!(ws.add_wire(n10, n11, &mut keygen), Some(AddWireResult::NoJoin(k0)));
        assert_eq!(ws.add_wire(n11, n12, &mut keygen), Some(AddWireResult::NoJoin(k0)));
        
        // Remove nodes
        assert_split(
            ws.remove_wire(n01, n11), k0, 
            n01, [n00, n01, n02], 
            n11, [n10, n11, n12]
        );

        // Check wire set was constructed correctly
        assert_graph_nodes(&ws.wires, nodes);
        assert_graph_edges(&ws.wires, [
            (k0, vec![(n00, n01), (n01, n02), (n10, n11), (n11, n12)]),
        ]);
        assert_range_map(&ws.ranges, [
            (n00, n01), (n01, n02),
            (n10, n11), (n11, n12),
        ]);
    }

    #[test]
    fn wireset_remove_joint_erase() {
        let mut keygen = keygen();
        let mut ws = WireSet::default();

        let [n00, n01, n02, n11] = [
            (0, 0), (0, 1), (0, 2), (1, 1),
        ];

        // Add nodes ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let Some(AddWireResult::NoJoin(key)) = ws.add_wire(n00, n01, &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins");
        };
        assert_eq!(ws.add_wire(n01, n11, &mut keygen), Some(AddWireResult::NoJoin(key)));
        assert_eq!(ws.add_wire(n01, n02, &mut keygen), Some(AddWireResult::NoJoin(key)));
        
        // Remove nodes
        assert_split(
            ws.remove_wire(n01, n11), key,
            n01, [n00, n01, n02],
            n11, [n11]
        );

        // Check wire set constructed correctly
        assert_graph_nodes(&ws.wires, [n00, n02]);
        assert_graph_edges(&ws.wires, [
            (key, vec![(n00, n02)])
        ]);
        assert_range_map(&ws.ranges, [(n00, n02)]);
    }

    #[test]
    fn wireset_remove_subset() {
        let mut keygen = keygen();
        let mut ws = WireSet::default();

        let nodes @ [n00, n01, n02, n03] = [
            (0, 0), (0, 1), (0, 2), (0, 3)
        ];

        // Test ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let Some(AddWireResult::NoJoin(key)) = ws.add_wire(n00, n03, &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins");
        };
        assert_split(
            ws.remove_wire(n01, n02), key,
            n01, [n00, n01],
            n02, [n02, n03]
        );

        // Check wire set constructed correctly
        assert_graph_nodes(&ws.wires, nodes);

        let edges = [(n00, n01), (n02, n03)];
        assert_graph_edges(&ws.wires, [(key, edges.to_vec())]);
        assert_range_map(&ws.ranges, edges);
    }

    #[test]
    fn wireset_remove_no_split() {
        let mut keygen = keygen();
        let mut ws = WireSet::default();

        let [n00, n01, n02, n10, n11, n12] = [
            (0, 0), (0, 4), (0, 8),
            (4, 0), (4, 4), (4, 8)
            ];

        // Add nodes:
        let Some(AddWireResult::NoJoin(key)) = ws.add_wire(n00, n01, &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins")
        };
        assert_eq!(ws.add_wire(n01, n11, &mut keygen), Some(AddWireResult::NoJoin(key)));
        assert_eq!(ws.add_wire(n11, n10, &mut keygen), Some(AddWireResult::NoJoin(key)));
        assert_eq!(ws.add_wire(n10, n00, &mut keygen), Some(AddWireResult::NoJoin(key)));
        assert_eq!(ws.add_wire(n11, n12, &mut keygen), Some(AddWireResult::NoJoin(key)));
        assert_eq!(ws.add_wire(n12, n02, &mut keygen), Some(AddWireResult::NoJoin(key)));
        assert_eq!(ws.add_wire(n02, n01, &mut keygen), Some(AddWireResult::NoJoin(key)));

        // Remove nodes:
        assert_eq!(ws.remove_wire(n01, n11), Some(RemoveWireResult::NoSplit(key)));
    }
}