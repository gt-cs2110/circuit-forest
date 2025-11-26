use std::collections::{BTreeMap, HashMap, HashSet};

use petgraph::Undirected;
use petgraph::prelude::GraphMap;
use petgraph::visit::{Bfs, Walker};

use crate::circuit::graph::ValueKey;
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
pub enum AddWireResult {
    /// No joining is necessary.
    /// The [`ValueKey`] provided is the key of the added wire.
    NoJoin(ValueKey),

    /// Joining is necessary.
    /// The parameters are:
    /// - The coordinate to start a flood fill from.
    /// - The value key the new wire is set to.
    /// - The value key which needs to be replaced with the new wire's key.
    Join(Coord, ValueKey, ValueKey)
}

/// The result type for [`WireSet::remove_wire`].
/// 
/// This enum indicates whether a [`ValueKey`] must be split into two.
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

/// The connection of wires in a circuit.
#[derive(Debug, Default)]
pub struct WireSet {
    wires: GraphMap<MeshKey, ValueKey, Undirected>,
    horiz_wires: HashMap<Axis, BTreeMap<Axis, Axis>>,
    vert_wires: HashMap<Axis, BTreeMap<Axis, Axis>>
}
impl WireSet {
    /// Find the ValueKey corresponding to a coordinate (if it exists).
    pub fn find_key(&self, p: Coord) -> Option<ValueKey> {
        self.wires.edges(p.into())
            .next()
            .map(|(_, _, &k)| k)
    }

    /// Finds the farthest coordinate from `c`
    /// which can be used to create a 1D wire of the specified orientation.
    /// 
    /// This assumes coordinate `c` is a wire with one neighbor.
    fn max_wire_endpoint(&self, c: Coord, horizontal: bool) -> Coord {
        let mut parent = c;
        let mut child = c;
        loop {
            let mut it = self.wires.edges(child.into())
                // Get all physical wires
                .filter_map(|(m1, m2, _)| match (m1, m2) {
                    (MeshKey::WireJoint(p), MeshKey::WireJoint(q)) => Some((q, Wire::from_endpoints(p, q)?)),
                    _ => None
                })
                // Find any wires which match the same orientation and would not create the same wire as (parent-child)
                .filter(|(q, w)| w.horizontal == horizontal && q != &parent);

            match (it.next(), it.next()) {
                (Some((next_pt, _)), None) => (parent, child) = (child, next_pt),
                _ => break
            };
        }

        child
    }

    /// Checks if there's a wire at the coordinate and splitting it into two if needed.
    /// 
    /// Returns whether a split was successful.
    fn split_joint(&mut self, c: Coord, horizontal: bool) -> bool {
        let splittable_wires = match horizontal {
            true  => self.vert_wires_at_coord(c),
            false => self.horiz_wires_at_coord(c),
        };
        
        let mut it = splittable_wires
            .into_iter()
            .filter_map(|w| Some((w, w.split(c)?)));
        
        let Some((w, [w1, w2])) = it.next() else {
            return false;
        };
        debug_assert!(it.next().is_none(), "Expected only one splittable wire");

        // Split wires in graph:
        let [p, q] = w.endpoints();
        let Some(k) = self.wires.remove_edge(p.into(), q.into()) else {
            unreachable!("Expected wire to split to exist");
        };
        self.wires.add_edge(p.into(), c.into(), k);
        self.wires.add_edge(c.into(), q.into(), k);
        // Split wires in map:
        match w1.horizontal {
            true  => self.horiz_wires.entry(w1.y).or_default().insert(w1.x, w1.length),
            false => self.vert_wires.entry(w1.x).or_default().insert(w1.y, w1.length)
        };
        match w2.horizontal {
            true  => self.horiz_wires.entry(w2.y).or_default().insert(w2.x, w2.length),
            false => self.vert_wires.entry(w2.x).or_default().insert(w2.y, w2.length)
        };

        true
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
    pub fn add_wire(&mut self, p: Coord, q: Coord, new_vk: impl FnOnce() -> ValueKey) -> Option<AddWireResult> {
        Wire::from_endpoints(p, q).map(|w| {
            // If horizontal or vertical, these two points can be connected.
    
            // TODO: Detect if intersecting another wire
    
            // Add to wire maps:
            match w.horizontal {
                true  => self.horiz_wires.entry(w.y).or_default().insert(w.x, w.length),
                false => self.vert_wires.entry(w.x).or_default().insert(w.y, w.length)
            };
    
            // Add to wire graph:
            let [p, q] = w.endpoints();
            match (self.find_key(p), self.find_key(q)) {
                (None, None) => {
                    let new_key = new_vk();
                    self.wires.add_edge(p.into(), q.into(), new_key);
                    AddWireResult::NoJoin(new_key)
                },
                (Some(key), None) | (None, Some(key)) => {
                    self.wires.add_edge(p.into(), q.into(), key);
                    AddWireResult::NoJoin(key)
                },
                (Some(pk), Some(qk)) if pk == qk => {
                    self.wires.add_edge(p.into(), q.into(), pk);
                    AddWireResult::NoJoin(pk)
                },
                (Some(pk), Some(qk)) => {
                    self.wires.add_edge(p.into(), q.into(), pk);
                    AddWireResult::Join(q, pk, qk)
                }
            }
        })
    }
    
    /// Removes the wire from the graph between p and q.
    /// 
    /// Note that this function only removes wires that are directly connected by joints
    /// in the circuit.
    /// 
    /// If this function returns None, the wire does not exist & could not be removed.
    /// Otherwise, this function returns data needed to split a [`ValueKey`] (if applicable).
    pub fn remove_wire(&mut self, p: Coord, q: Coord) -> Option<RemoveWireResult> {
        let [p, q] = minmax(p, q);
        
        // Remove from wire graph:
        let e = self.wires.remove_edge(p.into(), q.into())?;

        // Remove from wire map:
        let Some(w) = Wire::from_endpoints(p, q) else {
            unreachable!("({p:?}, {q:?}) must constitute a valid wire due to being successfully removed from the graph");
        };
        let m_len = match w.horizontal {
            true  => self.horiz_wires.entry(w.y).or_default().remove(&w.x),
            false => self.vert_wires.entry(w.x).or_default().remove(&w.y)
        };
        debug_assert_eq!(m_len, Some(w.length), "Expected wire to be removed from map with correct length");

        // If removed, also check if a ValueKey needs to be split
        let joints: HashSet<_> = Bfs::new(&self.wires, q.into())
            .iter(&self.wires)
            .filter_map(|m| match m {
                MeshKey::WireJoint(c) => Some(c),
                MeshKey::Tunnel(_) => None,
            })
            .collect();
        
        // Remove extraneous, disconnected nodes
        if joints.is_empty() {
            self.wires.remove_node(q.into());
        }
        if self.wires.neighbors(p.into()).next().is_some() {
            self.wires.remove_node(p.into());
        }

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
            }
        }
    }

    fn vert_wires_at_coord(&self, c: Coord) -> Vec<Wire> {
        let Some(m) = self.vert_wires.get(&c.0) else { return vec![] };
        m.range(..=c.1).rev()
            .map(|(&y, &length)| Wire { x: c.0, y, length, horizontal: false })
            .take_while(|w| w.contains(c))
            .collect()
    }
    fn horiz_wires_at_coord(&self, c: Coord) -> Vec<Wire> {
        let Some(m) = self.horiz_wires.get(&c.1) else { return vec![] };
        m.range(..=c.0).rev()
            .map(|(&x, &length)| Wire { x, y: c.1, length, horizontal: true })
            .take_while(|w| w.contains(c))
            .collect()
    }

    /// Gets all wire segments coinciding at the specified coords.
    /// 
    /// This returns all wire segments, including segments that this coord
    /// is in the middle of.
    pub fn wires_at_coord(&self, c: Coord) -> Vec<Wire> {
        let mut wires = self.horiz_wires_at_coord(c);
        wires.extend(self.vert_wires_at_coord(c));
        wires
    }
}

fn minmax(p: Coord, q: Coord) -> [Coord; 2] {
    if q < p { [q, p] } else { [p, q] }
}

/// A wire.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash, serde::Serialize, serde::Deserialize)]
pub struct Wire {
    /// The lowermost X coordinate of the wire.
    x: Axis,
    /// The lowermost Y coordinate of the wire.
    y: Axis,
    /// The length of the wire.
    length: Axis,
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

        acceptable.then_some(Self { x, y, length, horizontal })
    }

    /// Constructs a wire out of endpoints, returning None if not 1D.
    pub fn from_endpoints(p: Coord, q: Coord) -> Option<Self> {
        // Let p = the left-/top-most coord, q = the other coord.
        let [p, q] = minmax(p, q);

        match (q.0 - p.0, q.1 - p.1) {
            (0, 0) => None,
            (0, length) => Some(Self { x: p.0, y: p.1, length, horizontal: true }),
            (length, 0) => Some(Self { x: p.0, y: p.1, length, horizontal: false }),
            _ => None
        }
    }

    /// The endpoints of the wire.
    pub fn endpoints(&self) -> [Coord; 2] {
        match self.horizontal {
            true  => [(self.x, self.y), (self.x + self.length, self.y)],
            false => [(self.x, self.y), (self.x, self.y + self.length)],
        }
    }

    /// Detect whether this wire includes the specified coordinate.
    pub fn contains(&self, c: Coord) -> bool {
        match self.horizontal {
            true  => self.y == c.1 && (self.x <= c.0) && (c.0 - self.x <= self.length),
            false => self.x == c.0 && (self.y <= c.1) && (c.1 - self.y <= self.length),
        }
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