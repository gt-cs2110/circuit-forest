use std::collections::HashSet;

use petgraph::prelude::UnGraphMap;
use petgraph::visit::{Bfs, Walker};

use crate::engine::{FunctionPort, ValueKey};
use crate::middle_end::string_interner::TunnelSymbol;
use crate::middle_end::wire::{Wire, WireRangeMap};
use crate::middle_end::{ComponentKey, Coord};


/// A key to attach onto the wire set graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MeshKey {
    /// A joint (a wire point)
    WireJoint(Coord),
    /// A component port (excluding tunnels).
    Port(ComponentKey, usize),
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
        let FunctionPort { gate, index } = value;
        Self::Port(gate.into(), index)
    }
}
impl From<(ComponentKey, usize)> for MeshKey {
    fn from(value: (ComponentKey, usize)) -> Self {
        let (component, index) = value;
        Self::Port(component, index)
    }
}
impl From<TunnelSymbol> for MeshKey {
    fn from(value: TunnelSymbol) -> Self {
        Self::Tunnel(value)
    }
}

type WireGraph = UnGraphMap<MeshKey, ValueKey>;
/// The connection of wires in a circuit.
#[derive(Debug, Default)]
pub struct WireSet {
    graph: WireGraph,
    ranges: WireRangeMap,
}

/// Handles various operations with [`Valuekey`]s
/// that need to be handled outside of the wire set.
pub trait ValueFinalizer {
    /// Create a new value key.
    fn gen_key(&mut self) -> ValueKey;
    /// Delete the current value key.
    fn delete_key(&mut self, k: ValueKey);
    /// Recomputes and updates the value for the given value key.
    fn update_key(&mut self, k: ValueKey);
    /// Joins the objects linked to the specified keys into the [`into`] key.
    fn join(&mut self, into: ValueKey, keys: &[ValueKey]);
    /// Splits the objects linked into a new value key.
    fn split(&mut self, key: ValueKey, split_off: &[(ComponentKey, usize)]) -> ValueKey;
    /// Connects a port to a given value key.
    fn connect_port(&mut self, gate: ComponentKey, index: usize, key: ValueKey);
}

impl WireSet {
    /// Find the [`ValueKey`] corresponding to a coordinate.
    /// 
    /// This is `None` if the coordinate is not connected to a wire.
    pub fn find_key<K: Into<MeshKey>>(&self, key: K) -> Option<ValueKey> {
        let mut edges = self.graph.edges(key.into())
            .map(|(_, _, &k)| k);
        let next = edges.next();

        debug_assert!(
            next.is_none_or(|next| edges.all(|k| next == k)),
            "Mesh should only have one key"
        );

        next
    }

    /// Finds the [`ValueKey`] corresponding to a given wire.
    /// 
    /// This is `None` if the wire is not present on the graph.
    /// The two endpoints must be joints (i.e., the specified wire cannot be a subwire).
    pub fn find_key_of_wire(&self, w: Wire) -> Option<ValueKey> {
        let [p, q] = w.endpoints();
        self.graph.edge_weight(p.into(), q.into()).copied()
    }

    /// Tries to split the wire facing the specified horizontality
    /// at the point of where `c` is.
    /// 
    /// This function will successfully split if there is a wire at `c`
    /// which can be broken up into two parts on both sides of `c`.
    fn split_wire_on_joint(&mut self, c: Coord, horizontal: bool) {
        if let Some(joined) = self.ranges.split_wire(horizontal, c) {
            let [p, q] = joined.endpoints();
            // Split wires in graph:
            let Some(k) = self.graph_remove_wire(joined) else {
                unreachable!("Expected wire to split to exist");
            };
            self.graph.add_edge(p.into(), c.into(), k);
            self.graph.add_edge(c.into(), q.into(), k);
        }
    }

    /// Splits all wires at a given coord.
    fn split_at_coord(&mut self, c: Coord) {
        self.split_wire_on_joint(c, true);
        self.split_wire_on_joint(c, false);
    }

    /// Performs a join at the given coord,
    /// allowing two wires at the given point to be joined
    /// if it would make sense for the geometry of the graph.
    fn join_at_coord(&mut self, c: Coord) {
        self.join_at_coord_with(c, |lk, rk| {
            assert_eq!(lk, rk, "Joined wires should have same keys");
            lk
        });
    }
    /// Performs a join at the given coord,
    /// allowing two wires at the given point to be joined
    /// if it would make sense for the geometry of the graph.
    fn join_at_coord_with(&mut self, c: Coord, merge: impl FnOnce(ValueKey, ValueKey) -> ValueKey) {
        let mut neighbors = self.graph.neighbors(c.into());

        // Exactly two neighbors, two of which are wire joints, which form a straight wire
        if let Some(MeshKey::WireJoint(p)) = neighbors.next()
            && let Some(MeshKey::WireJoint(q)) = neighbors.next()
            && neighbors.next().is_none()
            && let Some(joined) = Wire::from_endpoints(p, q)
        {
            let left = Wire::from_endpoints(p, c).expect("graph edge should've formed valid wire");
            let right = Wire::from_endpoints(c, q).expect("graph edge should've formed valid wire");
            // Update graph:
            let lk = self.graph_remove_wire(left).expect("removable wire");
            let rk = self.graph_remove_wire(right).expect("removable wire");
            self.graph.add_edge(p.into(), q.into(), merge(lk, rk));
            // Update ranges:
            assert!(self.ranges.remove_wire(left));
            assert!(self.ranges.remove_wire(right));
            assert!(self.ranges.add_wire(joined));
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

    /// Add a wire to the graph.
    /// 
    /// This function may create or join value keys,
    /// requiring a [`ValueFinalizer`] to handle those cases.
    /// 
    /// This function may add additional wires (e.g., if a connection would result in an intersection)
    /// or subsume wires which already exist (e.g., to extend a wire).
    /// 
    /// If this function returns None, the wire could not be added.
    /// Otherwise, this function returns the value key of the new wire.
    pub fn add_wire(&mut self, w: Wire, vf: &mut impl ValueFinalizer) -> Option<ValueKey> {
        let [p, q] = w.endpoints();

        let removed_wires: Vec<_> = self.ranges
            .parallel_overlapping_wires(w)
            .collect();

        // Check there's actually something to add.
        // If the overlap range envelops the wire's range and all wires connect,
        //    then there's nothing to add.
        let range = match *removed_wires {
            [] => None,
            [w] => Some(w.endpoints()),
            [wl, .., wr] => Some([wl.endpoints()[0], wr.endpoints()[1]])
        };
        // Overlap range envelops wire's range & all wires connect
        if
            let Some([l, r]) = range
            && l <= p && q <= r
            && removed_wires.array_windows().all(|[lw, rw]| lw.endpoints()[1] == rw.endpoints()[0])
        {
            return None;
        }

        let [p, q] = range.map_or([p, q], |[l, r]| [l.min(p), r.max(q)]);
        let w = Wire::from_endpoints(p, q)
            .unwrap_or_else(|| unreachable!("new wire should've existed"));

        // If endpoint intersects a perpendicular wire, make a joint:
        self.split_wire_on_joint(p, !w.horizontal);
        self.split_wire_on_joint(q, !w.horizontal);

        // Get all the intersections and keys.
        // Note that after wire removal, some of the intersection points may disappear.
        //     In this case, we'll remove the intersections later,
        //     but we should ensure we have all the keys.
        let (mut intersections, keys): (Vec<_>, HashSet<_>) = w.coord_iter()
            .filter_map(|c| Some((c, self.find_key(c)?)))
            .collect();

        // Delete each wire in the range.
        for &w in &removed_wires {
            let removed = self.graph_remove_wire(w);
            debug_assert!(
                removed.is_some_and(|k| keys.contains(&k)),
                "Removal of edge should have value key which is already accounted for"
            );

            let removed = self.ranges.remove_wire(w);
            debug_assert!(removed);
        }
        intersections.retain(|&c| self.graph.contains_node(c.into()));

        // Update keys for all joining wires.
        let keys = Vec::from_iter(keys);
        let (fill_key, rest_keys) = match keys.split_first() {
            Some((&m, rest)) => (m, rest),
            None => (vf.gen_key(), &[][..])
        };
        self.flood_fill(&intersections, fill_key);
        vf.join(fill_key, rest_keys);

        // Add the wire, split it, and check if the edges need to be joined.
        self.graph.add_edge(p.into(), q.into(), fill_key);
        let added = self.ranges.add_wire(w);
        debug_assert!(added);
        for ix in intersections {
            self.split_wire_on_joint(ix, w.horizontal);
        }

        self.join_at_coord_with(p, |_, _| fill_key);
        self.join_at_coord_with(q, |_, _| fill_key);
        
        Some(fill_key)
    }
    
    /// Adds a port to the graph, connecting some coordinate to the port.
    /// 
    /// This function may create keys, requiring a [`ValueFinalizer`] to handle those cases.
    ///
    /// If addition was not possible (e.g., if edge already exists or if port already exists as a node),
    /// this returns None. Otherwise, this returns the value key of the new port.
    pub fn add_port(&mut self, c: Coord, key: ComponentKey, index: usize, vf: &mut impl ValueFinalizer) -> Option<ValueKey> {
        let port = (key, index).into();

        if self.graph.contains_node(port) {
            return None;
        }

        if self.graph.contains_edge(c.into(), port) {
            return None;
        }

        self.split_at_coord(c); // If point is in middle of wire, split it
        
        let vk = self.find_key(c).unwrap_or_else(|| vf.gen_key());
        self.graph.add_edge(c.into(), port, vk);
        vf.connect_port(key, index, vk);

        Some(vk)
    }

    /// Adds a tunnel link to the graph, connecting some coordinate to the tunnel.
    /// 
    /// /// This function may create or join value keys,
    /// requiring a [`ValueFinalizer`] to handle those cases.
    /// 
    /// A `new_vk` callback needs to be provided in case the edge is disconnected 
    /// from the rest of the graph and needs a new key.
    /// 
    /// If addition was not possible (e.g., if edge already exists),
    /// this returns None. Otherwise, this returns the value key of the new port.
    pub fn add_tunnel(&mut self, c: Coord, tunnel: TunnelSymbol, vf: &mut impl ValueFinalizer) -> Option<ValueKey> {
        if self.graph.contains_edge(c.into(), tunnel.into()) {
            return None;
        }

        self.split_at_coord(c); // If point is in middle of wire, split it

        // Get the key of the new coordinate (if it exists) and get the key of the tunnel.
        let added_key = self.find_key(c);
        let tunnel_key = self.find_key(tunnel);

        let edge_key = match (added_key, tunnel_key) {
            (None, None) => vf.gen_key(),
            (None, Some(k)) | (Some(k), None) => k,
            (Some(added), Some(tunnel)) => {
                // If coordinate key doesn't match the tunnel key,
                // convert it to the tunnel key.
                if added != tunnel {
                    self.flood_fill(&[c], tunnel);
                    vf.join(tunnel, &[added]);
                }

                tunnel
            },
        };

        self.graph.add_edge(c.into(), tunnel.into(), edge_key);
        Some(edge_key)
    }

    /// Removes the wire from the graph.
    /// 
    /// Note that this function only removes wires that are directly connected by joints
    /// in the circuit.
    /// 
    /// If this function returns None, the wire does not exist & could not be removed.
    /// Otherwise, this function returns data needed to split a [`ValueKey`] (if applicable).
    #[must_use]
    pub fn remove_wire(&mut self, w: Wire, vf: &mut impl ValueFinalizer) -> bool {
        let [p, q] = w.endpoints();
        
        self.split_wire_on_joint(p, w.horizontal);
        self.split_wire_on_joint(q, w.horizontal);

        let removed: Vec<_> = self.ranges
            .parallel_overlapping_wires(w)
            .collect();
        // Nothing to remove, so no need to continue:
        if removed.is_empty() {
            return false;
        }

        let mut deleted_keys = HashSet::new();
        for &w in &removed {
            let k = self.graph_remove_wire(w).expect("key should be deleted");
            deleted_keys.insert(k);

            let result = self.ranges.remove_wire(w);
            debug_assert!(result);
        }

        // Get any points attached to the removed wire.
        // Since we're going to join right after this, make sure to have an extra point
        //    in case the wire gets joined.
        let mut coords = Vec::from_iter(w.coord_iter());
        coords.extend({
            w.coord_iter().filter_map(|c| match self.graph.neighbors(c.into()).next()? {
                MeshKey::WireJoint(w) => Some(w),
                _ => None
            })
        });
        // Join any wires with excess splits.
        for w in removed {
            let [l, r] = w.endpoints();
            self.join_at_coord(l);
            self.join_at_coord(r);
        }
        coords.retain(|&c| self.graph.contains_node(c.into()));
        
        // Split the value keys for wire meshes that have been split:
        let mut traversed_keys = HashSet::new();
        while let Some(c) = coords.pop() {
            let group_set: HashSet<_> = Bfs::new(&self.graph, c.into())
                .iter(&self.graph)
                .collect();
            coords.retain(|&c| !group_set.contains(&c.into()));

            let k = self.find_key(c)
                .expect("coordinate should have an assigned key");
            
            deleted_keys.remove(&k);
            // Split key if duplicate:
            if !traversed_keys.insert(k) {
                let group: Vec<_> = group_set.iter()
                    .filter_map(|&mk| match mk {
                        MeshKey::Port(k, i) => Some((k, i)),
                        _ => None
                    })
                    .collect();
                
                let split_key = vf.split(k, &group);
                self.flood_fill(&[c], split_key);
            }
        }
        for k in deleted_keys {
            vf.delete_key(k);
        }

        true
    }

    /// Removes a port from the graph.
    /// 
    /// If this function returns `None`, then the port doesn't exist on the graph.
    /// If this function returns `Some(_)`, it returns a `RemoveWireResult`,
    ///     which may include a key to delete.
    #[must_use]
    pub fn remove_port(&mut self, key: ComponentKey, index: usize, vf: &mut impl ValueFinalizer) -> bool {
        let port = (key, index).into();
        let mut it = self.graph.neighbors(port);
        
        let Some(MeshKey::WireJoint(c)) = it.next() else {
            return false;
        };
        debug_assert!(it.next().is_none(), "Function port should only have 1 neighbor");

        let Some(k) = self.graph_remove_edge(port, c.into()) else {
            return false;
        };
        debug_assert!(!self.graph.contains_node(port), "Function port should no longer exist");

        match self.graph.contains_node(c.into()) {
            // Disconnect means key needs to be updated.
            true => vf.update_key(k),
            // If coord no longer exists, key cannot exist.
            false => vf.delete_key(k),
        }

        self.join_at_coord(c);
        true
    }

    /// Removes a tunnel link from the graph.
    /// 
    /// If this function returns `None`, then the edge doesn't exist on the graph.
    /// If the function returns `Some(_)`, it returns a `RemoveWireResult`,
    ///     which may indicate keys to delete & split.
    #[must_use]
    pub fn remove_tunnel(&mut self, c: Coord, tunnel: TunnelSymbol, vf: &mut impl ValueFinalizer) -> bool {
        let Some(k) = self.graph_remove_edge(c.into(), tunnel.into()) else {
            return false;
        };

        // If neither node exists, then the key of this link can no longer exist.
        let port_exists = self.graph.contains_node(c.into());
        let tun_exists = self.graph.contains_node(tunnel.into());
        if !port_exists && !tun_exists {
            vf.delete_key(k);
        }
        
        // If the port still exists, then we need to split the key between the tunnel and port meshes.
        // If the port no longer exists, then there was nothing on the other side.
        // Tunnel doesn't have any update logic, so nothing needs to be updated.
        if port_exists {
            let ports: Vec<_> = Bfs::new(&self.graph, c.into())
                .iter(&self.graph)
                .filter_map(|mk| match mk {
                    MeshKey::Port(key, index) => Some((key, index)),
                    _ => None
                })
                .collect();
            
            let split_key = vf.split(k, &ports);
            self.flood_fill(&[c], split_key);
        }
        
        self.join_at_coord(c);
        true
    }

    /// Replaces the [`ValueKey`] of all wires connecting to the Coord
    /// with the specified flood key.
    /// 
    /// All wires with a path to the coordinate that are not of the flood key
    /// are replaced with the flood key.
    /// 
    /// This returns the set of keys which were traversed.
    fn flood_fill(&mut self, entry_points: &[Coord], flood_key: ValueKey) -> Vec<MeshKey> {
        let mut frontier: Vec<MeshKey> = entry_points.iter()
            .map(|&c| c.into())
            .collect();
        let mut consumed = 0;

        while consumed < frontier.len() {
            let k = frontier[consumed];
            consumed += 1;

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

        frontier
    }

    /// Gets all wire segments coinciding at the specified coords.
    /// 
    /// This returns all wire segments, including segments that this coord
    /// is in the middle of.
    pub fn wires_at_coord(&self, c: Coord) -> impl Iterator<Item = Wire> {
        self.ranges.wires_at_coord(c)
    }

    /// Gets all of the wires defined in the set.
    pub fn wires(&self) -> impl Iterator<Item=Wire> {
        self.ranges.wires()
    }

    /// Gets all of the wires in the set and their associated value keys.
    pub fn wire_values_iter(&self) -> impl Iterator<Item=(Wire, ValueKey)> {
        self.graph.all_edges()
            .filter_map(|(m1, m2, &vk)| match (m1, m2) {
                (MeshKey::WireJoint(p), MeshKey::WireJoint(q)) => Some((Wire::from_endpoints(p, q).unwrap(), vk)),
                _ => None
            })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashMap};

    use slotmap::SlotMap;

    use crate::middle_end::string_interner::StringInterner;
    use crate::middle_end::wire::range_map::assert_range_map;

    use super::*;
    
    struct Keygen<K: slotmap::Key>(SlotMap<K, ()>);
    impl<K: slotmap::Key> Keygen<K> {
        fn new() -> Self {
            Self(Default::default())
        }
        fn gen_key(&mut self) -> K {
            self.0.insert(())
        }
        fn len(&self) -> usize {
            self.0.len()
        }
        fn is_empty(&self) -> bool {
            self.0.is_empty()
        }
    }
    impl ValueFinalizer for Keygen<ValueKey> {
        fn gen_key(&mut self) -> ValueKey {
            self.gen_key()
        }
        fn delete_key(&mut self, k: ValueKey) {
            self.0.remove(k);
        }
        fn update_key(&mut self, _: ValueKey) {}
        fn join(&mut self, _: ValueKey, keys: &[ValueKey]) {
            for &k in keys {
                self.0.remove(k);
            }
        }
        fn split(&mut self, _: ValueKey, _: &[(ComponentKey, usize)]) -> ValueKey {
            self.gen_key()
        }
        
        fn connect_port(&mut self, _: ComponentKey, _: usize, _: ValueKey) {}
    }

    /// Asserts nodes of the graph are exactly the specified node list.
    fn assert_graph_nodes<const N: usize>(graph: &WireGraph, nodes: [Coord; N]) {
        let actual: BTreeSet<_> = graph.nodes().collect();
        let expected: BTreeSet<_> = nodes.into_iter().map(Into::into).collect();
        assert_eq!(actual, expected, "nodes in graph should match");
    }
    fn assert_graph_edges<const N: usize, K>(graph: &WireGraph, all_edges: [(ValueKey, Vec<(K, K)>); N])
        where K: Into<MeshKey> + Copy
    {
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
    fn assert_graph_edges_unkeyed<const N: usize, K>(graph: &WireGraph, all_edges: [Vec<(K, K)>; N])
        where K: Into<MeshKey> + Copy
    {
        use crate::middle_end::wire::minmax;
        
        let expected_edgemap: HashSet<_> = all_edges.into_iter()
            .map(|edgeset| {
                let mut set: Vec<_> = edgeset.into_iter()
                    .map(|(l, r)| minmax(l.into(), r.into()))
                    .collect();
                set.sort();

                set
            })
            .collect();
        
        let mut edgelist: Vec<_> = graph.all_edges().collect();
        edgelist.sort_by_key(|&(_, _, &vk)| vk);
        let actual_edgemap: HashSet<_> = edgelist.chunk_by(|(_, _, vk0), (_, _, vk1)| vk0 == vk1)
            .map(|edgeset| {
                let mut set: Vec<_> = edgeset.iter()
                    .map(|&(a, b, _)| minmax(a, b))
                    .collect();
                set.sort();

                set
            })
            .collect();

        assert_eq!(actual_edgemap, expected_edgemap, "edges did not match");
    }
    fn w(p: Coord, q: Coord) -> Wire {
        Wire::from_endpoints(p, q)
            .expect("points should be 1D")
    }

    /// Assert edges of the graph are exactly the specified edge list.
    #[test]
    fn wireset_add_basic() {
        let mut keygen = Keygen::new();
        let mut ws = WireSet::default();

        let nodes @ [n00, n01, n11, n12, n02] = [(0, 0), (0, 4), (4, 4), (4, 10), (0, 10)];

        // Add nodes:
        let key = ws.add_wire(w(n00, n01), &mut keygen)
            .expect("Expected first wire add to be successful and require no joins");
        assert_eq!(ws.add_wire(w(n01, n11), &mut keygen), Some(key));
        assert_eq!(ws.add_wire(w(n11, n12), &mut keygen), Some(key));
        assert_eq!(ws.add_wire(w(n01, n02), &mut keygen), Some(key));

        // Check wire set was constructed correctly
        assert_graph_nodes(&ws.graph, nodes);

        let edges = [(n00, n01), (n01, n11), (n11, n12), (n01, n02)];
        assert_graph_edges(&ws.graph, [(key, edges.to_vec())]);
        assert_range_map(&ws.ranges, edges);
    }

    #[test]
    fn wireset_add_duplicate() {
        let mut keygen = Keygen::new();
        let mut ws = WireSet::default();

        assert!(ws.add_wire(w((0, 0), (0, 1)), &mut keygen).is_some());
        assert!(ws.add_wire(w((0, 0), (0, 1)), &mut keygen).is_none()); // same wire
    }

    #[test]
    fn wireset_add_join() {
        let mut keygen = Keygen::new();
        let mut ws = WireSet::default();

        let nodes @ [n00, n01, n02, n10, n11, n12] = [
            (2, 2), (2, 3), (1, 3),
            (3, 4), (3, 3), (4, 3),
        ];

        // Add nodes ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let k0 = ws.add_wire(w(n00, n01), &mut keygen)
            .expect("Expected first wire add to be successful and require no joins");
        assert_eq!(ws.add_wire(w(n01, n02), &mut keygen), Some(k0));
        
        let k1 = ws.add_wire(w(n10, n11), &mut keygen)
            .expect("Expected second wire add to be successful and require no joins");
        assert_eq!(ws.add_wire(w(n11, n12), &mut keygen), Some(k1));
        
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
        let join_key = ws.add_wire(w(n01, n11), &mut keygen)
            .expect("Expected wire addition to succeed");

        // Check wire set was constructed correctly
        assert_graph_nodes(&ws.graph, nodes);
        assert_graph_edges(&ws.graph, [(
                join_key, vec![
                (n00, n01), (n01, n02),
                (n10, n11), (n11, n12),
                (n01, n11)
            ]
        )]);
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

        let mut keygen = Keygen::new();
        let mut ws = WireSet::default();

        let [n00, n01, n02, n03, n04, n05, n13] = [
            (1, 1), (3, 1), (5, 1), (7, 1), (9, 1), (11, 1),
            (7, 3),
        ];

        // Add nodes (1) ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let key = ws.add_wire(w(n01, n02), &mut keygen)
            .expect("Expected first wire add to be successful and require no joins");
        assert_eq!(ws.add_wire(w(n02, n03), &mut keygen), Some(key));
        assert_eq!(ws.add_wire(w(n00, n01), &mut keygen), Some(key));
        
        // Check wire set was constructed correctly
        assert_graph_nodes(&ws.graph, [n00, n03]);
        
        assert_graph_edges(&ws.graph, [
            (key, vec![(n00, n03)]),
        ]);
        assert_range_map(&ws.ranges, [(n00, n03)]);

        // Add nodes (2) ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        assert_eq!(ws.add_wire(w(n13, n03), &mut keygen), Some(key));
        assert_eq!(ws.add_wire(w(n03, n04), &mut keygen), Some(key));
        assert_eq!(ws.add_wire(w(n04, n05), &mut keygen), Some(key));

        // Check wire set was constructed correctly
        assert_graph_nodes(&ws.graph, [n00, n03, n13, n05]);
        
        let edges = [(n00, n03), (n03, n13), (n03, n05)];
        assert_graph_edges(&ws.graph, [(key, edges.to_vec())]);
        assert_range_map(&ws.ranges, edges);
    }

    #[test]
    fn wireset_add_subset() {
        let mut keygen = Keygen::new();
        let mut ws = WireSet::default();

        let [n00, n01, n02, n03] = [
            (1, 1), (3, 1), (5, 1), (7, 1)
        ];

        let key = ws.add_wire(w(n00, n02), &mut keygen)
            .expect("Expected first wire add to be successful and require no joins");
        assert_eq!(ws.add_wire(w(n01, n03), &mut keygen), Some(key));
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
        let mut keygen = Keygen::new();
        let mut ws = WireSet::default();

        let [n00, n01, n02, n11] = [
            (1, 1), (3, 1), (5, 1), (3, 3),
        ];

        // Add nodes (1) ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let key = ws.add_wire(w(n00, n02), &mut keygen)
            .expect("Expected first wire add to be successful and require no joins");
        assert_eq!(ws.add_wire(w(n01, n11), &mut keygen), Some(key));

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
        let mut keygen = Keygen::new();
        let mut ws = WireSet::default();

        let [n00, n01, n02, n11] = [
            (1, 1), (3, 1), (5, 1), (3, 3),
        ];

        // Add nodes (1) ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let key = ws.add_wire(w(n01, n11), &mut keygen)
            .expect("Expected first wire add to be successful and require no joins");
        assert_eq!(ws.add_wire(w(n00, n02), &mut keygen), Some(key));

        // Check wire set was constructed correctly
        assert_graph_nodes(&ws.graph, [n00, n01, n02, n11]);

        let edges = [(n00, n01), (n01, n02), (n01, n11)];
        assert_graph_edges(&ws.graph, [(key, edges.to_vec())]);
        assert_range_map(&ws.ranges, edges);
    }
    
    #[test]
    fn wireset_remove_basic() {
        let mut keygen = Keygen::new();
        let mut ws = WireSet::default();

        let [n00, n01, n11, n12, n02] = [(0, 0), (0, 4), (4, 4), (4, 10), (0, 10)];

        // Add nodes:
        let key = ws.add_wire(w(n00, n01), &mut keygen)
            .expect("Expected first wire add to be successful and require no joins");
        assert_eq!(ws.add_wire(w(n01, n11), &mut keygen), Some(key));
        assert_eq!(ws.add_wire(w(n11, n12), &mut keygen), Some(key));
        assert_eq!(ws.add_wire(w(n01, n02), &mut keygen), Some(key));

        // Remove nodes:
        assert!(ws.remove_wire(w(n01, n02), &mut keygen));
        assert!(ws.remove_wire(w(n11, n12), &mut keygen));
        assert!(ws.remove_wire(w(n01, n11), &mut keygen));
        assert!(ws.remove_wire(w(n00, n01), &mut keygen));

        // Check correct construction
        assert_graph_nodes(&ws.graph, []);
        assert_graph_edges::<_, MeshKey>(&ws.graph, []);
        assert_range_map(&ws.ranges, []);
        assert!(keygen.is_empty());
    }

    #[test]
    fn wireset_remove_overlong() {
        let mut keygen = Keygen::new();
        let mut ws = WireSet::default();

        let [n0, n1, n2] = [(0, 1), (0, 2), (0, 3)];
        
        ws.add_wire(w(n0, n1), &mut keygen)
            .expect("Expected first wire add to be successful and require no joins");
        
        assert!(ws.remove_wire(w(n0, n2), &mut keygen));
        assert_graph_nodes(&ws.graph, []);
        assert_graph_edges_unkeyed::<_, MeshKey>(&ws.graph, []);
        assert_range_map(&ws.ranges, []);
        assert!(keygen.is_empty());
    }

    #[test]
    fn wireset_remove_overlong2() {
        let mut keygen = Keygen::new();
        let mut ws = WireSet::default();

        let [n0, n1, n2, n3, n4, n5] = [
            (0, 0), (0, 1), (0, 2), (0, 3), (0, 4), (0, 5)
        ];
        ws.add_wire(w(n1, n2), &mut keygen)
            .expect("Expected first wire add to be successful and require no joins");
        ws.add_wire(w(n3, n4), &mut keygen)
            .expect("Expected first wire add to be successful and require no joins");
        
        assert!(ws.remove_wire(w(n0, n5), &mut keygen));
        assert_graph_nodes(&ws.graph, []);
        assert_graph_edges_unkeyed::<_, MeshKey>(&ws.graph, []);
        assert_range_map(&ws.ranges, []);
        assert!(keygen.is_empty());
    }

    #[test]
    fn wireset_remove_fail() {
        let mut keygen = Keygen::new();
        let mut ws = WireSet::default();

        assert!(!ws.remove_wire(w((0, 0), (0, 1)), &mut keygen)); // Empty

        let _ = ws.add_wire(w((0, 1), (0, 2)), &mut keygen)
            .expect("Expected first wire add to be successful and require no joins");
        assert!(!ws.remove_wire(w((0, 5), (0, 9)), &mut keygen)); // Does not exist
    }

    #[test]
    fn wireset_remove_split() {
        let mut keygen = Keygen::new();
        let mut ws = WireSet::default();

        let nodes @ [n00, n01, n02, n10, n11, n12] = [
            (2, 2), (2, 3), (1, 3),
            (3, 4), (3, 3), (4, 3),
        ];

        // Add nodes ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let k0 = ws.add_wire(w(n00, n01), &mut keygen)
            .expect("Expected first wire add to be successful and require no joins");
        assert_eq!(ws.add_wire(w(n01, n02), &mut keygen), Some(k0));
        assert_eq!(ws.add_wire(w(n01, n11), &mut keygen), Some(k0));
        assert_eq!(ws.add_wire(w(n10, n11), &mut keygen), Some(k0));
        assert_eq!(ws.add_wire(w(n11, n12), &mut keygen), Some(k0));
        
        // Remove nodes
        assert!(ws.remove_wire(w(n01, n11), &mut keygen));

        // Check wire set was constructed correctly
        assert_graph_nodes(&ws.graph, nodes);
        assert_graph_edges_unkeyed(&ws.graph, [
            vec![(n00, n01), (n01, n02)],
            vec![(n10, n11), (n11, n12)],
        ]);
        assert_range_map(&ws.ranges, [
            (n00, n01), (n01, n02),
            (n10, n11), (n11, n12),
        ]);
        assert_eq!(keygen.len(), 2);
    }

    #[test]
    fn wireset_remove_joint_erase() {
        let mut keygen = Keygen::new();
        let mut ws = WireSet::default();

        let [n00, n01, n02, n11] = [
            (0, 0), (0, 1), (0, 2), (1, 1),
        ];

        // Add nodes ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let key = ws.add_wire(w(n00, n01), &mut keygen)
            .expect("Expected first wire add to be successful and require no joins");
        assert_eq!(ws.add_wire(w(n01, n11), &mut keygen), Some(key));
        assert_eq!(ws.add_wire(w(n01, n02), &mut keygen), Some(key));
        
        // Remove nodes
        assert!(ws.remove_wire(w(n01, n11), &mut keygen));

        // Check wire set constructed correctly
        assert_graph_nodes(&ws.graph, [n00, n02]);
        assert_graph_edges(&ws.graph, [
            (key, vec![(n00, n02)])
        ]);
        assert_range_map(&ws.ranges, [(n00, n02)]);
        assert_eq!(keygen.len(), 1);
    }

    #[test]
    fn wireset_remove_subset() {
        let mut keygen = Keygen::new();
        let mut ws = WireSet::default();

        let nodes @ [n00, n01, n02, n03] = [
            (0, 0), (0, 1), (0, 2), (0, 3)
        ];

        // Test ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        ws.add_wire(w(n00, n03), &mut keygen)
            .expect("Expected first wire add to be successful and require no joins");
        assert!(ws.remove_wire(w(n01, n02), &mut keygen));

        // Check wire set constructed correctly
        assert_graph_nodes(&ws.graph, nodes);

        assert_graph_edges_unkeyed(&ws.graph, [
            vec![(n00, n01)], vec![(n02, n03)]
        ]);
        assert_range_map(&ws.ranges, [(n00, n01), (n02, n03)]);
        assert_eq!(keygen.len(), 2);
    }

    #[test]
    fn wireset_remove_no_split() {
        let mut keygen = Keygen::new();
        let mut ws = WireSet::default();

        let [n00, n01, n02, n10, n11, n12] = [
            (0, 0), (0, 4), (0, 8),
            (4, 0), (4, 4), (4, 8)
            ];

        // Add nodes:
        let key = ws.add_wire(w(n00, n01), &mut keygen)
            .expect("Expected first wire add to be successful and require no joins");
        assert_eq!(ws.add_wire(w(n01, n11), &mut keygen), Some(key));
        assert_eq!(ws.add_wire(w(n11, n10), &mut keygen), Some(key));
        assert_eq!(ws.add_wire(w(n10, n00), &mut keygen), Some(key));
        assert_eq!(ws.add_wire(w(n11, n12), &mut keygen), Some(key));
        assert_eq!(ws.add_wire(w(n12, n02), &mut keygen), Some(key));
        assert_eq!(ws.add_wire(w(n02, n01), &mut keygen), Some(key));

        // Remove nodes:
        assert!(ws.remove_wire(w(n01, n11), &mut keygen));
        assert_eq!(keygen.len(), 1);
    }

    #[test]
    fn wireset_remove_slice_two() {
        let mut keygen = Keygen::new();
        let mut ws = WireSet::default();

        let [n00, n01, n02, n03, n04, n05] = [
            (0, 0), (0, 1), (0, 2), (0, 3), (0, 4), (0, 5)
        ];

        // Test ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let k1 = ws.add_wire(w(n00, n02), &mut keygen)
            .expect("Expected first wire add to be successful and require no joins");
        let k2 = ws.add_wire(w(n03, n05), &mut keygen)
            .expect("Expected second wire add to be successful and require no joins");
        assert_ne!(k1, k2);
        
        assert!(ws.remove_wire(w(n01, n04), &mut keygen));

        // Check wire set constructed correctly
        assert_graph_nodes(&ws.graph, [n00, n01, n04, n05]);

        let edges = [(n00, n01), (n04, n05)];
        assert_graph_edges(&ws.graph, [(k1, edges[..1].to_vec()), (k2, edges[1..].to_vec())]);
        assert_range_map(&ws.ranges, edges);
        assert_eq!(keygen.len(), 2);
    }

    #[test]
    fn wireset_remove_intersect() {
        let mut keygen = Keygen::new();
        let mut ws = WireSet::default();

        let [n01, n02, n10, n11, n12, n13, n21, n22] = [
                    (0, 1), (0, 2),
            (1, 0), (1, 1), (1, 2), (1, 3),
                    (2, 1), (2, 2),
        ];

        // Add nodes ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let k0 = ws.add_wire(w(n11, n12), &mut keygen)
            .expect("Expected first wire add to be successful and require no joins");
        assert_eq!(ws.add_wire(w(n01, n11), &mut keygen), Some(k0));
        assert_eq!(ws.add_wire(w(n11, n21), &mut keygen), Some(k0));
        assert_eq!(ws.add_wire(w(n02, n12), &mut keygen), Some(k0));
        assert_eq!(ws.add_wire(w(n12, n22), &mut keygen), Some(k0));
        assert_eq!(ws.add_wire(w(n10, n11), &mut keygen), Some(k0));
        assert_eq!(ws.add_wire(w(n12, n13), &mut keygen), Some(k0));
        
        // Remove nodes
        assert!(ws.remove_wire(w(n10, n13), &mut keygen));

        // Check wire set was constructed correctly
        assert_graph_nodes(&ws.graph, [n01, n21, n02, n22]);
        assert_graph_edges_unkeyed(&ws.graph, [
            vec![(n01, n21)], vec![(n02, n22)],
        ]);
        assert_range_map(&ws.ranges, [
            (n01, n21),
            (n02, n22),
        ]);
        assert_eq!(keygen.len(), 2);
    }

    #[test]
    fn wireset_mid_port() {
        let mut value_keygen = Keygen::new();
        let mut func_keygen = Keygen::new();
        let mut ws = WireSet::default();

        let [n00, n01, n02] = [
            (0, 0), (0, 1), (0, 2)
        ];
        let gate = func_keygen.gen_key();
        let port = FunctionPort { gate, index: 0 };
        // Test ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // Add a wire and put a port in the middle of it.
        let k1 = ws.add_wire(w(n00, n02), &mut value_keygen)
            .expect("Expected first wire add to be successful and require no joins");
        let k2 = ws.add_port(n01, port.gate.into(), port.index, &mut value_keygen)
            .expect("Expected port creation to succeed");

        assert_eq!(k1, k2);
        assert_graph_edges::<_, MeshKey>(&ws.graph, [(k1, vec![
            (n00.into(), n01.into()), (n01.into(), n02.into()), (n01.into(), port.into())
        ])]);
        assert_range_map(&ws.ranges, [(n00, n01), (n01, n02)]);

        // Remove one of the split wires and readd it.
        assert!(ws.remove_wire(w(n01, n02), &mut value_keygen));
        let k3 = ws.add_wire(w(n01, n02), &mut value_keygen)
            .expect("Expected second wire add to be successful and require no joins");
        assert_eq!(k1, k3);
        // Adding wires should be the same because the port still exists.
        assert_graph_edges::<_, MeshKey>(&ws.graph, [(k1, vec![
            (n00.into(), n01.into()), (n01.into(), n02.into()), (n01.into(), port.into())
        ])]);
        assert_range_map(&ws.ranges, [(n00, n01), (n01, n02)]);


        // Remove the port.
        assert!(ws.remove_port(port.gate.into(), port.index, &mut value_keygen));
        assert_graph_edges(&ws.graph, [(k1, vec![(n00, n02)])]);
        assert_range_map(&ws.ranges, [(n00, n02)]);
    }
    #[test]
    fn wireset_mid_port_2() {
        let mut value_keygen = Keygen::new();
        let mut func_keygen = Keygen::new();
        let mut ws = WireSet::default();

        let [n00, n01, n02] = [
            (0, 0), (0, 1), (0, 2)
        ];
        let gate = func_keygen.gen_key();
        let port = FunctionPort { gate, index: 0 };
        // Test ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // Add a port and put a wire in the middle of it
        
        let k2 = ws.add_port(n01, port.gate.into(), port.index, &mut value_keygen)
            .expect("Expected port creation to succeed");
        let k1 = ws.add_wire(w(n00, n02), &mut value_keygen)
            .expect("Expected first wire add to be successful and require no joins");

        assert_eq!(k1, k2);
        assert_graph_edges::<_, MeshKey>(&ws.graph, [(k1, vec![
            (n00.into(), n01.into()), (n01.into(), n02.into()), (n01.into(), port.into())
        ])]);
        assert_range_map(&ws.ranges, [(n00, n01), (n01, n02)]);

        // Remove one of the split wires and readd it.
        assert!(ws.remove_wire(w(n01, n02), &mut value_keygen));
        let k3 = ws.add_wire(w(n01, n02), &mut value_keygen)
            .expect("Expected second wire add to be successful and require no joins");
        assert_eq!(k1, k3);
        // Adding wires should be the same because the port still exists.
        assert_graph_edges::<_, MeshKey>(&ws.graph, [(k1, vec![
            (n00.into(), n01.into()), (n01.into(), n02.into()), (n01.into(), port.into())
        ])]);
        assert_range_map(&ws.ranges, [(n00, n01), (n01, n02)]);


        // Remove the port.
        assert!(ws.remove_port(port.gate.into(), port.index, &mut value_keygen));
        assert_graph_edges(&ws.graph, [(k1, vec![(n00, n02)])]);
        assert_range_map(&ws.ranges, [(n00, n02)]);
    }

    #[test]
    fn wireset_useless_tunnel() {
        let mut value_keygen = Keygen::new();
        let mut tunnel_gen = StringInterner::default();
        let mut ws = WireSet::default();

        let [n00, n01, n10, n11] = [(1, 1), (1, 2), (3, 2), (3, 3)];
        
        // Setup:
        let tun = tunnel_gen.add_ref("tunnel!!!");
        let k1 = ws.add_tunnel(n00, tun, &mut value_keygen)
            .expect("tunnel creation to succeed");
        let k2 = ws.add_tunnel(n11, tun, &mut value_keygen)
            .expect("tunnel creation to succeed");
        assert_eq!(k1, k2);

        let k3 = ws.add_wire(Wire::from_endpoints(n00, n01).unwrap(), &mut value_keygen)
            .expect("wire creation to succeed");
        let k4 = ws.add_wire(Wire::from_endpoints(n10, n11).unwrap(), &mut value_keygen)
            .expect("wire creation to succeed");
        assert_eq!(k1, k3);
        assert_eq!(k3, k4);

        // Connect meshes:
        let k5 = ws.add_wire(Wire::from_endpoints(n01, n10).unwrap(), &mut value_keygen)
            .expect("wire creation to succeed");
        assert_eq!(k1, k5);

        // Delete tunnel:
        assert!(ws.remove_tunnel(n00, tun, &mut value_keygen));
        assert_eq!(value_keygen.len(), 1);
    }
}