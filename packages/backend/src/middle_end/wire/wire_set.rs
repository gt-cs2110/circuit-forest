use std::collections::{HashMap, HashSet};

use petgraph::prelude::UnGraphMap;
use petgraph::visit::{Bfs, Walker};

use crate::circuit::graph::{FunctionPort, ValueKey};
use crate::middle_end::string_interner::TunnelSymbol;
use crate::middle_end::wire::{Wire, WireRangeMap};
use crate::middle_end::Coord;


/// A key to attach onto the wire set graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MeshKey {
    /// A joint (a wire point)
    WireJoint(Coord),
    /// A function port.
    Port(FunctionPort),
    /// A tunnel.
    Tunnel(TunnelSymbol),

}
impl From<Coord> for MeshKey {
    fn from(value: Coord) -> Self {
        Self::WireJoint(value)
    }
}
impl From<FunctionPort> for MeshKey {
    fn from(value: FunctionPort) -> Self {
        Self::Port(value)
    }
}
impl From<TunnelSymbol> for MeshKey {
    fn from(value: TunnelSymbol) -> Self {
        Self::Tunnel(value)
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

type SplitGroupMap = HashMap<ValueKey, Vec<HashSet<MeshKey>>>;
/// The result type for [`WireSet::remove_wire`].
/// 
/// The struct holds the keys that no longer have an edge associated
/// and keys that need to split.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct RemoveWireResult {
    /// Keys that need to be deleted (no edges are associated with it anymore).
    pub deleted_keys: HashSet<ValueKey>,
    /// A map of all keys and sets that need to be split.
    pub split_groups: SplitGroupMap
}

type WireGraph = UnGraphMap<MeshKey, ValueKey>;
/// The connection of wires in a circuit.
#[derive(Debug, Default)]
pub struct WireSet {
    graph: WireGraph,
    ranges: WireRangeMap,
}
impl WireSet {
    /// Find the ValueKey corresponding to a coordinate.
    /// 
    /// This is None if the coordinate is not connected to a wire.
    pub fn find_key<K: Into<MeshKey>>(&self, key: K) -> Option<ValueKey> {
        self.graph.edges(key.into())
            .next()
            .map(|(_, _, &k)| k)
    }

    /// Checks if there is a wire to split, and splitting it into two if needed.
    /// 
    /// This function accepts the coordinate to split at.
    fn split_wire_on_joint(&mut self, c: Coord, horizontal: bool) {
        if let Some((_, joined)) = self.ranges.split_wire(horizontal, c) {
            let [p, q] = joined.endpoints();
            // Split wires in graph:
            let Some(k) = self.graph_remove_wire(joined) else {
                unreachable!("Expected wire to split to exist");
            };
            self.graph.add_edge(p.into(), c.into(), k);
            self.graph.add_edge(c.into(), q.into(), k);
        }
    }

    
    /// Removes an edge from the graph and removes any singleton nodes.
    fn graph_remove_edge(&mut self, l: MeshKey, r: MeshKey) -> Option<ValueKey> {
        /// Removes node if it is not connected to any wire.
        fn remove_if_singleton(graph: &mut WireGraph, n: MeshKey) {
            if graph.neighbors(n).next().is_none() {
                graph.remove_node(n);
            }
        }

        let result = self.graph.remove_edge(l, r);
        remove_if_singleton(&mut self.graph, l);
        remove_if_singleton(&mut self.graph, r);

        result
    }

    /// Removes wire from graph, returning the value key if the wire was successfully removed.
    fn graph_remove_wire(&mut self, w: Wire) -> Option<ValueKey> {
        let [p, q] = w.endpoints();
        self.graph_remove_edge(p.into(), q.into())
    }

    /// Takes a group of keys and tries to separate each coordinate into ValueKey groups.
    ///
    /// This ignores any keys which no longer have an associated ValueKey.
    fn compute_meshes<K: Into<MeshKey>>(&mut self, coords: impl IntoIterator<Item=K>) -> SplitGroupMap {
        let mut split_groups = HashMap::new();
        
        let mut coords: Vec<_> = coords.into_iter()
            .map(|c| c.into())
            .filter_map(|c| Some((c, self.find_key(c)?)))
            .collect();

        // Find all groups of joints following split:
        while let Some((c, k)) = coords.pop() {
            let group: HashSet<_> = Bfs::new(&self.graph, c)
                .iter(&self.graph)
                .collect();

            coords.retain(|&(c, _)| !group.contains(&c));
            split_groups.entry(k)
                .or_insert_with(Vec::new)
                .push(group);
        }
        
        split_groups
    }
    /// Add a wire to the graph.
    /// A `new_vk` callback needs to be provided in case the wire is not
    /// connected to the rest of the graph and needs a new key.
    /// 
    /// This function may add additional wires (e.g., if a connection would result in an intersection)
    /// or subsume wires which already exist (e.g., to extend a wire).
    /// 
    /// If this function returns None, the wire could not be added.
    /// Otherwise, this function returns data needed to merge two groups of wires
    /// with different [`ValueKey`]s (if applicable).
    #[must_use]
    pub fn add_wire(&mut self, w: Wire, new_vk: impl FnOnce() -> ValueKey) -> Option<AddWireResult> {
        // If horizontal or vertical, these two points can be connected.
        let [p, q] = w.endpoints();
        let pk = self.find_key(p);
        let qk = self.find_key(q);

        // If endpoints intersect the middle of a wire, create an intersection:
        self.split_wire_on_joint(p, !w.horizontal);
        self.split_wire_on_joint(q, !w.horizontal);
        // All keys along wire:
        let keys: HashSet<_> = w.coord_iter()
            .filter_map(|c| self.find_key(c))
            .collect();

        // Add to wire maps:
        let mut removed = vec![];
        let mut added = vec![];
        for subwire in self.ranges.add_wire(w) {
            // For the two endpoints, try merging wires,
            // and keeping track of which wires are added/removed
            added.push(subwire);
            let [l, r] = subwire.endpoints();
            if let Some(([spl, spr], joined)) = self.ranges.join_wire(l) {
                // Remove spl:
                if added.pop_if(|&mut w| w == spl).is_none() {
                    removed.push(spl);
                }
                // Remove spr:
                let result = added.pop();
                debug_assert_eq!(result, Some(spr));
                // Add joined:
                added.push(joined);
            }
            if let Some(([spl, spr], joined)) = self.ranges.join_wire(r) {
                // Remove spl:
                let result = added.pop();
                debug_assert_eq!(result, Some(spl));
                // Remove spr, add joined:
                removed.push(spr);
                added.push(joined);
            }
        }
        if removed.is_empty() && added.is_empty() {
            return None;
        }

        // Delete each wire in `removed`.
        for w in removed {
            let removed = self.graph_remove_wire(w);
            debug_assert!(
                removed.is_some_and(|k| keys.contains(&k)),
                "Removal of edge should have value key which is already accounted for"
            );
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
        self.graph.extend(
            added.into_iter().map(|w| {
                let [l, r] = w.endpoints();
                (l.into(), r.into(), fill_key)
            })
        );
        // Break up any new wires with any joints that connect to this wire.
        for c in w.coord_iter() {
            let intersecting = self.graph.neighbors(c.into())
                .any(|other| matches!(other, MeshKey::WireJoint(_)));
            if intersecting {
                self.split_wire_on_joint(c, w.horizontal);
            }
        }

        Some(result)
    }
    
    /// Adds a port to the graph, connecting some coordinate to the port.
    /// A `new_vk` callback needs to be provided in case the edge is disconnected 
    /// from the rest of the graph and needs a new key.
    /// 
    /// This returns `Some(())` if addition was possible, or `None` if not
    /// (e.g., if edge already exists or if port already exists as a node).
    pub fn add_port(&mut self, c: Coord, port: FunctionPort, new_vk: impl FnOnce() -> ValueKey) -> Option<ValueKey> {
        if self.graph.contains_node(port.into()) {
            return None;
        }
        if self.graph.contains_edge(c.into(), port.into()) {
            return None;
        }

        let key = self.find_key(c).unwrap_or_else(new_vk);
        self.graph.add_edge(c.into(), port.into(), key);
        Some(key)
    }

    /// Adds a tunnel link to the graph, connecting some coordinate to the tunnel.
    /// A `new_vk` callback needs to be provided in case the edge is disconnected 
    /// from the rest of the graph and needs a new key.
    /// 
    /// This returns `Some(())` if addition was possible, or `None` if not
    /// (e.g., if edge already exists).
    pub fn add_tunnel(&mut self, c: Coord, tunnel: TunnelSymbol, new_vk: impl FnOnce() -> ValueKey) -> Option<ValueKey> {
        if self.graph.contains_edge(c.into(), tunnel.into()) {
            return None;
        }

        let key = self.find_key(c).unwrap_or_else(new_vk);
        self.graph.add_edge(c.into(), tunnel.into(), key);
        Some(key)
    }

    /// Removes the wire from the graph.
    /// 
    /// Note that this function only removes wires that are directly connected by joints
    /// in the circuit.
    /// 
    /// If this function returns None, the wire does not exist & could not be removed.
    /// Otherwise, this function returns data needed to split a [`ValueKey`] (if applicable).
    #[must_use]
    pub fn remove_wire(&mut self, w: Wire) -> Option<RemoveWireResult> {
        let [p, q] = w.endpoints();
        
        // Remove from wire graph & map:
        let (removed, added) = self.ranges.remove_wire(w);
        // No activity occurred, so no need to continue:
        if removed.is_empty() && added.is_empty() {
            return None;
        }

        for w in added {
            let [l, r] = w.endpoints();
            let k = self.find_key(l)
                .or_else(|| self.find_key(r))
                .expect("Added wire should have corresponding key");
            self.graph.add_edge(l.into(), r.into(), k);
        }

        let mut deleted_keys = HashSet::new();
        for w in removed {
            let [l, r] = w.endpoints();

            self.split_wire_on_joint(l, w.horizontal);
            self.split_wire_on_joint(r, w.horizontal);

            let k = self.graph_remove_wire(w).expect("Key should be deleted");
            deleted_keys.insert(k);
        }

        // Determine how the old key is split:
        let mut split_groups = self.compute_meshes(w.coord_iter());
        split_groups.retain(|k, groups| {
            deleted_keys.remove(k);
            groups.len() > 1 // if <= 1, then this key doesn't need to be split
        });

        // See if any edges can be joined:
        for c in [p, q] {
            if let Some(([l, r], j)) = self.ranges.join_wire(c) {
                let lk = self.graph_remove_wire(l).expect("removable wire");
                let rk = self.graph_remove_wire(r).expect("removable wire");
                assert_eq!(lk, rk, "Joined wires should have same keys");

                let [j0, j1] = j.endpoints();
                self.graph.add_edge(j0.into(), j1.into(), lk);
            }
        }

        Some(RemoveWireResult { deleted_keys, split_groups })
    }

    /// Removes a port from the graph.
    /// 
    /// If this function returns `None`, then the port doesn't exist on the graph.
    /// If this function returns `Some(_)`, it returns a `RemoveWireResult`,
    ///     which may include a key to delete.
    #[must_use]
    pub fn remove_port(&mut self, port: FunctionPort) -> Option<RemoveWireResult> {
        let p = port.into();
        let mut it = self.graph.neighbors(p);
        
        let MeshKey::WireJoint(c) = it.next()? else {
            return None;
        };
        debug_assert!(it.next().is_none(), "Function port should only have 1 neighbor");

        let k = self.graph_remove_edge(p, c.into())?;
        debug_assert!(!self.graph.contains_node(port.into()), "Function port should no longer exist");

        // If coord node no longer exists, 
        // then no wires are connected (and therefore this key cannot exist).
        let deleted_keys = match self.graph.contains_node(c.into()) {
            true  => HashSet::new(),
            false => HashSet::from([k]),
        };

        Some(RemoveWireResult { deleted_keys, split_groups: Default::default() })
    }

    /// Removes a tunnel link from the graph.
    /// 
    /// If this function returns `None`, then the edge doesn't exist on the graph.
    /// If the function returns `Some(_)`, it returns a `RemoveWireResult`,
    ///     which may indicate keys to delete & split.
    #[must_use]
    pub fn remove_tunnel(&mut self, c: Coord, tunnel: TunnelSymbol) -> Option<RemoveWireResult> {
        let k = self.graph_remove_edge(c.into(), tunnel.into())?;

        // If neither node exists, then the key of this link can no longer exist.
        let deleted_keys = match self.graph.contains_node(c.into()) || self.graph.contains_node(tunnel.into()) {
            true  => HashSet::new(),
            false => HashSet::from([k]),
        };
        // Find groups:
        let mut split_groups = self.compute_meshes::<MeshKey>([c.into(), tunnel.into()]);
        split_groups.retain(|_, groups| groups.len() > 1);

        Some(RemoveWireResult { deleted_keys, split_groups })
    }

    /// Replaces the [`ValueKey`] of all wires connecting to the Coord
    /// with the specified flood key.
    /// 
    /// All wires with a path to the coordinate that are not of the flood key
    /// are replaced with the flood key.
    pub(crate) fn flood_fill(&mut self, p: Coord, flood_key: ValueKey) {
        let mut frontier = vec![p.into()];

        while let Some(k) = frontier.pop() {
            let edges_to_flood: Vec<_> = self.graph.edges(k)
                .filter(|&(_, _, &key)| key != flood_key)
                .map(|(n1, n2, _)| (n1, n2))
                .collect();
            for (n1, n2) in edges_to_flood {
                if let Some(k) = self.graph.edge_weight_mut(n1, n2) {
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use slotmap::SlotMap;

    use crate::middle_end::wire::range_map::assert_range_map;

    use super::*;
    
    fn keygen() -> impl FnMut() -> ValueKey {
        let mut map = SlotMap::with_key();
        move || map.insert(())
    }
    /// Asserts nodes of the graph are exactly the specified node list.
    fn assert_graph_nodes<const N: usize>(graph: &WireGraph, nodes: [Coord; N]) {
        let actual: BTreeSet<_> = graph.nodes().collect();
        let expected: BTreeSet<_> = nodes.into_iter().map(Into::into).collect();
        assert_eq!(actual, expected, "nodes in graph should match");
    }
    fn assert_graph_edges<const N: usize>(graph: &WireGraph, all_edges: [(ValueKey, Vec<(Coord, Coord)>); N]) {
        use crate::middle_end::wire::minmax;
        
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
                .map(|&(l, r)| minmax(l.into(), r.into()))
                .collect();

            actual_edges.sort();
            expected_edges.sort();
            assert_eq!(actual_edges, expected_edges, "edges for key {key:?} should match")
        }
    }
    fn w(p: Coord, q: Coord) -> Wire {
        Wire::from_endpoints(p, q)
            .expect("points should be 1D")
    }

    /// Assert edges of the graph are exactly the specified edge list.
    #[test]
    fn wireset_add_basic() {
        let mut keygen = keygen();
        let mut ws = WireSet::default();

        let nodes @ [n00, n01, n11, n12, n02] = [(0, 0), (0, 4), (4, 4), (4, 10), (0, 10)];

        // Add nodes:
        let Some(AddWireResult::NoJoin(key)) = ws.add_wire(w(n00, n01), &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins")
        };
        assert_eq!(ws.add_wire(w(n01, n11), &mut keygen), Some(AddWireResult::NoJoin(key)));
        assert_eq!(ws.add_wire(w(n11, n12), &mut keygen), Some(AddWireResult::NoJoin(key)));
        assert_eq!(ws.add_wire(w(n01, n02), &mut keygen), Some(AddWireResult::NoJoin(key)));

        // Check wire set was constructed correctly
        assert_graph_nodes(&ws.graph, nodes);

        let edges = [(n00, n01), (n01, n11), (n11, n12), (n01, n02)];
        assert_graph_edges(&ws.graph, [(key, edges.to_vec())]);
        assert_range_map(&ws.ranges, edges);
    }

    #[test]
    fn wireset_add_duplicate() {
        let mut keygen = keygen();
        let mut ws = WireSet::default();

        assert!(matches!(ws.add_wire(w((0, 0), (0, 1)), &mut keygen), Some(AddWireResult::NoJoin(_))));
        assert!(ws.add_wire(w((0, 0), (0, 1)), &mut keygen).is_none()); // same wire
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
        let Some(AddWireResult::NoJoin(k0)) = ws.add_wire(w(n00, n01), &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins");
        };
        assert_eq!(ws.add_wire(w(n01, n02), &mut keygen), Some(AddWireResult::NoJoin(k0)));
        
        let Some(AddWireResult::NoJoin(k1)) = ws.add_wire(w(n10, n11), &mut keygen) else {
            panic!("Expected second wire add to be successful and require no joins");
        };
        assert_eq!(ws.add_wire(w(n11, n12), &mut keygen), Some(AddWireResult::NoJoin(k1)));
        
        // Check wire set was constructed correctly
        assert_graph_nodes(&ws.graph, nodes);

        let edges = [
            (n00, n01), (n01, n02),
            (n10, n11), (n11, n12)
        ];
        assert_graph_edges(&ws.graph, [
            (k0, edges[0..2].to_vec()),
            (k1, edges[2..4].to_vec()),
        ]);
        assert_range_map(&ws.ranges, edges);

        // Join ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let Some(AddWireResult::Join(ffpt, src_key, dst_key)) = ws.add_wire(w(n01, n11), &mut keygen) else {
            panic!("Expected join")
        };
        assert!(ffpt == n01 || ffpt == n11);
        assert!(src_key == k0 || src_key == k1);
        assert!(dst_key.contains(&k0) && dst_key.contains(&k1));

        // Check wire set was constructed correctly
        assert_graph_nodes(&ws.graph, nodes);

        assert_eq!(ws.graph.edge_count(), 5);
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
        let Some(AddWireResult::NoJoin(key)) = ws.add_wire(w(n01, n02), &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins");
        };
        assert_eq!(ws.add_wire(w(n02, n03), &mut keygen), Some(AddWireResult::NoJoin(key)));
        assert_eq!(ws.add_wire(w(n00, n01), &mut keygen), Some(AddWireResult::NoJoin(key)));
        
        // Check wire set was constructed correctly
        assert_graph_nodes(&ws.graph, [n00, n03]);
        
        assert_graph_edges(&ws.graph, [
            (key, vec![(n00, n03)]),
        ]);
        assert_range_map(&ws.ranges, [(n00, n03)]);

        // Add nodes (2) ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        assert_eq!(ws.add_wire(w(n13, n03), &mut keygen), Some(AddWireResult::NoJoin(key)));
        assert_eq!(ws.add_wire(w(n03, n04), &mut keygen), Some(AddWireResult::NoJoin(key)));
        assert_eq!(ws.add_wire(w(n04, n05), &mut keygen), Some(AddWireResult::NoJoin(key)));

        // Check wire set was constructed correctly
        assert_graph_nodes(&ws.graph, [n00, n03, n13, n05]);
        
        let edges = [(n00, n03), (n03, n13), (n03, n05)];
        assert_graph_edges(&ws.graph, [(key, edges.to_vec())]);
        assert_range_map(&ws.ranges, edges);
    }

    #[test]
    fn wireset_add_subset() {
        let mut keygen = keygen();
        let mut ws = WireSet::default();

        let [n00, n01, n02, n03] = [
            (1, 1), (3, 1), (5, 1), (7, 1)
        ];

        let Some(AddWireResult::NoJoin(key)) = ws.add_wire(w(n00, n02), &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins");
        };
        assert_eq!(ws.add_wire(w(n01, n03), &mut keygen), Some(AddWireResult::NoJoin(key)));
        // Nothing added:
        assert!(ws.add_wire(w(n00, n03), &mut keygen).is_none());
        assert!(ws.add_wire(w(n00, n02), &mut keygen).is_none());
        
        // Check wire set was constructed correctly
        assert_graph_nodes(&ws.graph, [n00, n03]);

        assert_graph_edges(&ws.graph, [
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
        let Some(AddWireResult::NoJoin(key)) = ws.add_wire(w(n00, n02), &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins");
        };
        assert_eq!(ws.add_wire(w(n01, n11), &mut keygen), Some(AddWireResult::NoJoin(key)));

        // Check wire set was constructed correctly
        assert_graph_nodes(&ws.graph, [n00, n01, n02, n11]);

        let edges = [(n00, n01), (n01, n02), (n01, n11)];
        assert_graph_edges(&ws.graph, [(key, edges.to_vec())]);
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
        let Some(AddWireResult::NoJoin(key)) = ws.add_wire(w(n01, n11), &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins");
        };
        assert_eq!(ws.add_wire(w(n00, n02), &mut keygen), Some(AddWireResult::NoJoin(key)));

        // Check wire set was constructed correctly
        assert_graph_nodes(&ws.graph, [n00, n01, n02, n11]);

        let edges = [(n00, n01), (n01, n02), (n01, n11)];
        assert_graph_edges(&ws.graph, [(key, edges.to_vec())]);
        assert_range_map(&ws.ranges, edges);
    }

    fn assert_remove(
        result: Option<RemoveWireResult>,
        e_deleted_keys: impl IntoIterator<Item=ValueKey>,
        e_split_groups: impl IntoIterator<Item=(ValueKey, Vec<HashSet<Coord>>)>
    ) {
        let Some(RemoveWireResult { deleted_keys, split_groups }) = result else {
            panic!("Expected removal to succeed");
        };

        let e_deleted_keys = e_deleted_keys.into_iter().collect();
        assert_eq!(deleted_keys, e_deleted_keys, "Expected deleted keys to match");

        // Fixed order for groups:
        let a_groups = split_groups.into_iter()
            .map(|(key, value)| (key, {
                let mut g: Vec<_> = value.into_iter()
                    .map(<BTreeSet<_>>::from_iter)
                    .collect();
                g.sort();
                g
            }))
            .collect::<std::collections::BTreeMap<_, _>>();

        let e_groups = e_split_groups.into_iter()
            .map(|(key, value)| (key, {
                let mut g: Vec<_> = value.into_iter()
                    .map(|s| {
                        s.into_iter()
                            .map(Into::into)
                            .collect::<BTreeSet<_>>()
                    })
                    .collect();
                g.sort();
                g
            }))
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(a_groups, e_groups, "Expected correct groups of nodes to split");
    }
    #[test]
    fn wireset_remove_basic() {
        let mut keygen = keygen();
        let mut ws = WireSet::default();

        let [n00, n01, n11, n12, n02] = [(0, 0), (0, 4), (4, 4), (4, 10), (0, 10)];

        // Add nodes:
        let Some(AddWireResult::NoJoin(key)) = ws.add_wire(w(n00, n01), &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins")
        };
        assert_eq!(ws.add_wire(w(n01, n11), &mut keygen), Some(AddWireResult::NoJoin(key)));
        assert_eq!(ws.add_wire(w(n11, n12), &mut keygen), Some(AddWireResult::NoJoin(key)));
        assert_eq!(ws.add_wire(w(n01, n02), &mut keygen), Some(AddWireResult::NoJoin(key)));

        // Remove nodes:
        assert_remove(ws.remove_wire(w(n01, n02)), [], []);
        assert_remove(ws.remove_wire(w(n11, n12)), [], []);
        assert_remove(ws.remove_wire(w(n01, n11)), [], []);
        assert_remove(ws.remove_wire(w(n00, n01)), [key], []);

        // Check corre0ct construction
        assert_graph_nodes(&ws.graph, []);
        assert_graph_edges(&ws.graph, []);
        assert_range_map(&ws.ranges, []);
    }

    #[test]
    fn wireset_remove_overlong() {
        let mut keygen = keygen();
        let mut ws = WireSet::default();

        let [n0, n1, n2] = [(0, 1), (0, 2), (0, 3)];
        let Some(AddWireResult::NoJoin(k)) = ws.add_wire(w(n0, n1), &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins")
        };

        assert_remove(ws.remove_wire(w(n0, n2)), [k], []);
    }

    #[test]
    fn wireset_remove_overlong2() {
        let mut keygen = keygen();
        let mut ws = WireSet::default();

        let [n0, n1, n2, n3, n4, n5] = [
            (0, 0), (0, 1), (0, 2), (0, 3), (0, 4), (0, 5)
        ];
        let Some(AddWireResult::NoJoin(k1)) = ws.add_wire(w(n1, n2), &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins")
        };
        let Some(AddWireResult::NoJoin(k2)) = ws.add_wire(w(n3, n4), &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins")
        };
        assert_remove(ws.remove_wire(w(n0, n5)), [k1, k2], []);
    }

    #[test]
    fn wireset_remove_fail() {
        let mut keygen = keygen();
        let mut ws = WireSet::default();

        assert_eq!(ws.remove_wire(w((0, 0), (0, 1))), None); // Empty

        let Some(AddWireResult::NoJoin(_)) = ws.add_wire(w((0, 1), (0, 2)), &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins")
        };
        assert_eq!(ws.remove_wire(w((0, 5), (0, 9))), None); // Does not exist
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
        let Some(AddWireResult::NoJoin(k0)) = ws.add_wire(w(n00, n01), &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins");
        };
        assert_eq!(ws.add_wire(w(n01, n02), &mut keygen), Some(AddWireResult::NoJoin(k0)));
        assert_eq!(ws.add_wire(w(n01, n11), &mut keygen), Some(AddWireResult::NoJoin(k0)));
        assert_eq!(ws.add_wire(w(n10, n11), &mut keygen), Some(AddWireResult::NoJoin(k0)));
        assert_eq!(ws.add_wire(w(n11, n12), &mut keygen), Some(AddWireResult::NoJoin(k0)));
        
        // Remove nodes
        assert_remove(
            ws.remove_wire(w(n01, n11)),
            [],
            [(k0, vec![HashSet::from([n00, n01, n02]), HashSet::from([n10, n11, n12])])]
        );

        // Check wire set was constructed correctly
        assert_graph_nodes(&ws.graph, nodes);
        assert_graph_edges(&ws.graph, [
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
        let Some(AddWireResult::NoJoin(key)) = ws.add_wire(w(n00, n01), &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins");
        };
        assert_eq!(ws.add_wire(w(n01, n11), &mut keygen), Some(AddWireResult::NoJoin(key)));
        assert_eq!(ws.add_wire(w(n01, n02), &mut keygen), Some(AddWireResult::NoJoin(key)));
        
        // Remove nodes
        assert_remove(ws.remove_wire(w(n01, n11)), [], []);

        // Check wire set constructed correctly
        assert_graph_nodes(&ws.graph, [n00, n02]);
        assert_graph_edges(&ws.graph, [
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
        let Some(AddWireResult::NoJoin(key)) = ws.add_wire(w(n00, n03), &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins");
        };
        assert_remove(
            ws.remove_wire(w(n01, n02)),
            [],
            [(key, vec![HashSet::from([n00, n01]), HashSet::from([n02, n03])])]
        );

        // Check wire set constructed correctly
        assert_graph_nodes(&ws.graph, nodes);

        let edges = [(n00, n01), (n02, n03)];
        assert_graph_edges(&ws.graph, [(key, edges.to_vec())]);
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
        let Some(AddWireResult::NoJoin(key)) = ws.add_wire(w(n00, n01), &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins")
        };
        assert_eq!(ws.add_wire(w(n01, n11), &mut keygen), Some(AddWireResult::NoJoin(key)));
        assert_eq!(ws.add_wire(w(n11, n10), &mut keygen), Some(AddWireResult::NoJoin(key)));
        assert_eq!(ws.add_wire(w(n10, n00), &mut keygen), Some(AddWireResult::NoJoin(key)));
        assert_eq!(ws.add_wire(w(n11, n12), &mut keygen), Some(AddWireResult::NoJoin(key)));
        assert_eq!(ws.add_wire(w(n12, n02), &mut keygen), Some(AddWireResult::NoJoin(key)));
        assert_eq!(ws.add_wire(w(n02, n01), &mut keygen), Some(AddWireResult::NoJoin(key)));

        // Remove nodes:
        assert_remove(ws.remove_wire(w(n01, n11)), [], []);
    }

    #[test]
    fn wireset_remove_slice_two() {
        let mut keygen = keygen();
        let mut ws = WireSet::default();

        let [n00, n01, n02, n03, n04, n05] = [
            (0, 0), (0, 1), (0, 2), (0, 3), (0, 4), (0, 5)
        ];

        // Test ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let Some(AddWireResult::NoJoin(k1)) = ws.add_wire(w(n00, n02), &mut keygen) else {
            panic!("Expected first wire add to be successful and require no joins");
        };
        let Some(AddWireResult::NoJoin(k2)) = ws.add_wire(w(n03, n05), &mut keygen) else {
            panic!("Expected second wire add to be successful and require no joins");
        };
        assert_ne!(k1, k2);
        
        assert_remove(ws.remove_wire(w(n01, n04)), [], []);

        // Check wire set constructed correctly
        assert_graph_nodes(&ws.graph, [n00, n01, n04, n05]);

        let edges = [(n00, n01), (n04, n05)];
        assert_graph_edges(&ws.graph, [(k1, edges[..1].to_vec()), (k2, edges[1..].to_vec())]);
        assert_range_map(&ws.ranges, edges);
    }
}