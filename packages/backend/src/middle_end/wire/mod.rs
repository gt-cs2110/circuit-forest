use std::collections::{BTreeMap, HashMap, HashSet};

use petgraph::Undirected;
use petgraph::prelude::GraphMap;
use petgraph::visit::{Bfs, Walker};

use crate::circuit::graph::ValueKey;
use crate::middle_end::{Axis, Coord, UIKey};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum MeshKey {
    WireJoint(Coord),
    Tunnel(UIKey)
}
impl From<Coord> for MeshKey {
    fn from(value: Coord) -> Self {
        MeshKey::WireJoint(value)
    }
}

pub enum AddResult {
    NoJoin(ValueKey),
    Join(Coord, ValueKey, ValueKey)
}
pub enum RemoveResult {
    NoSplit(ValueKey),
    Split(Coord, ValueKey, HashSet<Coord>)
}
#[derive(Debug, Default)]
pub struct WireSet {
    wires: GraphMap<MeshKey, ValueKey, Undirected>,
    horiz_wires: HashMap<Axis, BTreeMap<Axis, Axis>>,
    vert_wires: HashMap<Axis, BTreeMap<Axis, Axis>>
}
impl WireSet {
    pub fn find_key(&self, p: Coord) -> Option<ValueKey> {
        self.wires.edges(p.into())
            .next()
            .map(|(_, _, &k)| k)
    }

    pub fn add_wire(&mut self, p: Coord, q: Coord, new_vk: impl FnOnce() -> ValueKey) -> Option<AddResult> {
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
                    AddResult::NoJoin(new_key)
                },
                (Some(key), None) | (None, Some(key)) => {
                    self.wires.add_edge(p.into(), q.into(), key);
                    AddResult::NoJoin(key)
                },
                (Some(pk), Some(qk)) if pk == qk => {
                    self.wires.add_edge(p.into(), q.into(), pk);
                    AddResult::NoJoin(pk)
                },
                (Some(pk), Some(qk)) => {
                    self.wires.add_edge(p.into(), q.into(), pk);
                    AddResult::Join(q, pk, qk)
                }
            }
        })
    }
    
    pub fn remove_wire(&mut self, p: Coord, q: Coord) -> Option<RemoveResult> {
        let [p, q] = minmax(p, q);
        
        // Remove from wire graph:
        let e = self.wires.remove_edge(p.into(), q.into())?;
        // Remove from wire map:
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
        
        // TODO: Remove extraneous, disconnected nodes

        let result = match joints.contains(&p) {
            true  => RemoveResult::NoSplit(e),
            false => RemoveResult::Split(q, e, joints),
        };
        Some(result)
    }

    pub fn flood_fill(&mut self, p: Coord, flood_key: ValueKey) {
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