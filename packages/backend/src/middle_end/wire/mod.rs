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
        let [p, q] = minmax(p, q);
        let is_horiz = is_horiz(p, q);
        let is_vert = is_vert(p, q);

        (is_horiz || is_vert).then(|| {
            // If horizontal or vertical, these two points can be connected.

            // TODO: Detect if intersecting another wire

            // Add to wire maps:
            match is_horiz {
                true  => self.horiz_wires.entry(p.1).or_default().insert(p.0, q.0 - p.0),
                false => self.vert_wires.entry(p.0).or_default().insert(p.1, q.1 - p.1)
            };

            // Add to wire graph:
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
        // TODO: Debug assert size is correct
        if is_horiz(p, q) {
            self.horiz_wires.entry(p.1).or_default().remove(&p.0);
        } else if is_vert(p, q) {
            self.vert_wires.entry(p.0).or_default().remove(&p.1);
        }

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

    /// Gets all wire segments coinciding at the specified coords.
    /// 
    /// This returns all wire segments, including segments that this coord
    /// is in the middle of.
    pub fn wires_at_coord(&self, c: Coord) -> Vec<[Coord; 2]> {
        let mut wires = vec![];

        // Get all horizontal wires containing c
        if let Some(m) = self.horiz_wires.get(&c.1) {
            let it = m.range(..=c.0).rev()
                .map(|(&x, &length)| Wire { x, y: c.1, length, horizontal: true })
                .take_while(|w| w.contains(c))
                .map(|w| w.endpoints());
            wires.extend(it);
        }
        // Get all vertical wires containing c
        if let Some(m) = self.vert_wires.get(&c.0) {
            let it = m.range(..=c.1).rev()
                .map(|(&y, &length)| Wire { x: c.0, y, length, horizontal: true })
                .take_while(|w| w.contains(c))
                .map(|w| w.endpoints());
            wires.extend(it);
        }

        wires
    }
}

fn minmax(p: Coord, q: Coord) -> [Coord; 2] {
    if q < p { [q, p] } else { [p, q] }
}
fn is_horiz(p: Coord, q: Coord) -> bool {
    p.1 == q.1
}
fn is_vert(p: Coord, q: Coord) -> bool {
    p.0 == q.0
}

/// A wire.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash, serde::Serialize, serde::Deserialize)]
pub struct Wire {
    /// The lowermost X coordinate of the wire.
    pub x: Axis,
    /// The lowermost Y coordinate of the wire.
    pub y: Axis,
    /// The length of the wire.
    pub length: Axis,
    /// Whether the wire is horizontal or vertical.
    #[serde(rename = "isHorizontal")]
    pub horizontal: bool
}
impl Wire {
    /// Constructs a wire out of endpoints, returning None if not 1D.
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