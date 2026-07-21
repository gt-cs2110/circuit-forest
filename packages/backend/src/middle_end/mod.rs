//! The middle end, which keeps track of component types, properties, and positions.
//! 
//! The main structs are:
//! - [`MiddleRepr`]: The main middle-end circuit representation.
//! - [`MiddleCircuit`]: A mutable view of one of the middle-end circuits.
//! 

use slotmap::{SecondaryMap};
use thiserror::Error;

use crate::engine::{Circuit, CircuitForest, CircuitKey, CircuitState, FunctionPort, ValueKey};
use crate::middle_end::comp_key::ComponentMap;
use crate::middle_end::func::{AbsoluteComponentBounds, ComponentBounds, Orientation, PhysicalComponent, PhysicalComponentEnum, PhysicalInitContext, coord_add};
use crate::middle_end::string_interner::StringInterner;
use crate::middle_end::wire::{ValueFinalizer, Wire, WireSet};

mod comp_key;
mod string_interner;
#[cfg(feature="serde")]
pub mod serialize;
pub mod wire;
pub mod func;

pub use string_interner::TunnelSymbol;
pub use comp_key::{ComponentKey, UIKey};

type Axis = u32;
type Coord = (Axis, Axis);

type AxisDelta = i32;
type CoordDelta = (AxisDelta, AxisDelta);

/// A group of middle circuits.
#[derive(Default)]
#[cfg_attr(feature="serde", derive(serde::Deserialize), serde(try_from = "serialize::CircuitFile"))]
pub struct MiddleRepr {
    engine: CircuitForest,
    physical: SecondaryMap<CircuitKey, CircuitArea>
}
impl std::fmt::Debug for MiddleRepr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use crate::engine::debug::DebugMap;

        f.debug_struct("MiddleRepr")
            .field("engine", &self.engine)
            .field("physical", &DebugMap(&self.physical))
            .finish()
    }
}
/// A circuit's middle-end components and wires,
///   including their locations and properties.
#[derive(Debug, Default)]
struct CircuitArea {
    name: String,
    components: ComponentMap<ComponentProps>,
    wires: WireSet,
    tunnel_interner: StringInterner
}

/// Properties of a middle-end component.
#[derive(Debug)]
pub struct ComponentProps {
    /// The label for the component.
    pub label: String,
    /// The location of the label for the component.
    pub label_location: Orientation,

    /// The component's origin, which holds its fixed point.
    pub origin: Coord,
    /// The bounds of the component.
    pub bounds: [Coord; 2],
    /// The location of all ports for the component.
    pub ports: Vec<Coord>,

    /// Component-specific props.
    pub inner: PhysicalComponentEnum
}
impl ComponentProps {
    /// Constructs the args needed to create these props.
    pub fn as_args(&self) -> AddComponentArgs<'_> {
        AddComponentArgs {
            inner: self.inner,
            label: &self.label,
            label_location: self.label_location,
            origin: self.origin,
        }
    }
}

/// Arguments used to invoke [`add_component`] and related additive methods.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct AddComponentArgs<'label> {
    /// The inner component properties.
    pub inner: PhysicalComponentEnum,
    /// The component's label (or "" if no label exists)
    pub label: &'label str,
    /// The location of the component label.
    pub label_location: Orientation,
    /// The component's origin.
    pub origin: Coord
}
impl AddComponentArgs<'static> {
    /// Constructs an argument set assuming the component has no label.
    pub fn unlabeled<C: Into<PhysicalComponentEnum>>(inner: C, origin: Coord) -> Self {
        Self {
            inner: inner.into(),
            label: "",
            label_location: Orientation::North,
            origin
        }
    }
}
#[derive(Debug, Error)]
/// Errors which can occur when editing a middle-end circuit.
pub enum ReprEditErr {
    /// Component is out of bounds (so it cannot be added).
    #[error("component is out of bounds")]
    ComponentOutOfBounds,
    
    /// Component being specified doesn't exist (so it cannot be removed).
    #[error("component does not exist")]
    ComponentDoesNotExist,
    
    /// Adding a tunnel at a spot where the same tunnel already exists.
    #[error("cannot place two identical tunnels with the same port")]
    RedundantTunnel,
}

/// A mutable view of a middle-end circuit,
/// which includes its engine component ([`crate::engine::Circuit`])
/// and its physical properties.
#[derive(Debug)]
pub struct MiddleCircuit<'a> {
    repr: &'a mut MiddleRepr,
    key: CircuitKey
}
impl MiddleRepr {
    /// Creates a new middle representation.
    pub fn new() -> Self {
        Self::default()
    }
    /// Creates a new subcircuit.
    pub fn add_circuit(&mut self, name: &str) -> CircuitKey {
        let ck = self.engine.add_circuit();

        let area = CircuitArea {
            name: name.to_string(),
            ..Default::default()
        };
        self.physical.insert(ck, area);

        ck
    }

    /// Creates a mutable view for a given subcircuit,
    /// panicking if the circuit does not exist.
    pub fn circuit(&mut self, key: CircuitKey) -> MiddleCircuit<'_> {
        self.try_circuit(key)
            .unwrap_or_else(|| panic!("circuit of key {key:?} does not exist"))
    } 

    /// Tries to create a mutable view for a given subcircuit,
    /// returning None if the circuit does not exist.
    pub fn try_circuit(&mut self, key: CircuitKey) -> Option<MiddleCircuit<'_>> {
        match self.physical.contains_key(key) {
            true => Some(MiddleCircuit { repr: self, key }),
            false => None
        }
    }
}

/// Basic macro to pretend Circuit has the "graph" and "state" fields.
/// 
/// This cannot be done with a function
/// because this is returning a place rather than a value.
macro_rules! circ {
    ($self:ident.engine)   => { $self.repr.engine.circuit($self.key) };
    ($self:ident.graph)    => { $self.repr.engine.graphs[$self.key] };
    ($self:ident.state)    => { $self.repr.engine.states[$self.key] };
    ($self:ident.physical) => { $self.repr.physical[$self.key] };
}

impl ValueFinalizer for Circuit<'_> {
    fn gen_key(&mut self) -> ValueKey {
        self.add_value_node()
    }

    fn delete_key(&mut self, k: ValueKey) {
        self.remove_value_node(k);
    }

    fn update_key(&mut self, k: ValueKey) {
        self.update_key(k);
    }

    fn join(&mut self, into: ValueKey, keys: &[ValueKey]) {
        self.join(into, keys);
    }

    fn split(&mut self, key: ValueKey, split_off: &[(ComponentKey, usize)]) -> ValueKey {
        let split_off = split_off.iter()
            .filter_map(|&(c, index)| match c {
                ComponentKey::Function(gate) => Some(FunctionPort { gate, index }),
                ComponentKey::UI(_) => None,
            });
        self.split(key, split_off)
    }
    
    fn connect_port(&mut self, gate: ComponentKey, index: usize, key: ValueKey) {
        if let ComponentKey::Function(gate) = gate {
            self.connect_one(key, FunctionPort { gate, index });
        }
    }
}
impl MiddleCircuit<'_> {
    /// Gets the bounds of the component to be added by the specified arguments.
    fn get_component_bounds(&self, args: AddComponentArgs<'_>) -> Option<AbsoluteComponentBounds> {
        let AddComponentArgs { inner, label, origin, .. } = args;
        let ctx = PhysicalInitContext { circuit: self, label };
        inner.init_bounds(ctx)
            .into_absolute(origin)
    }

    /// Adds a component to the circuit.
    /// 
    /// This takes the component, label, and location for the component.
    /// This returns [`ReprEditErr::ComponentOutOfBounds`] if it fails, which can occur if the component would be out of bounds. Otherwise, return the component key associated with added component.
    pub fn add_component(&mut self, args: AddComponentArgs<'_>) -> Result<ComponentKey, ReprEditErr> {
        let ComponentBounds { bounds, ports } = self.get_component_bounds(args)
            .ok_or(ReprEditErr::ComponentOutOfBounds)?;
        
        let AddComponentArgs { inner, label, label_location, origin } = args;
        let props = ComponentProps {
            label: label.to_string(),
            label_location,
            origin,
            bounds,
            ports,
            inner,
        };

        let gate = inner.init_engine()
            .map(|func| circ!(self.engine).add_function_node(func));
        let key = circ!(self.physical).components.insert(gate, props);

        // Update the wire set to include all the component's ports.
        //    For tunnels, all tunnels are treated as one unified port.
        //    For every other type of component, each port is passed onto the wire set.
        let CircuitArea { name: _, components, wires, tunnel_interner } = &mut circ!(self.physical);
        let props = &components[key];
        if matches!(props.inner, PhysicalComponentEnum::Tunnel(_)) {
            if !props.label.is_empty() {
            // Add tunnel to wire set:
                let &[coord] = props.ports.as_slice() else { unreachable!("Tunnel should have 1 port") };
                let sym = tunnel_interner.add_ref(&props.label);
                wires.add_tunnel(coord, sym, &mut circ!(self.engine))
                    .ok_or(ReprEditErr::RedundantTunnel)?;
            }
        } else {
            // Add port to wire set:
            for (index, &c) in props.ports.iter().enumerate() {
                wires.add_port(c, key, index, &mut circ!(self.engine))
                    .expect("Expected port addition to be successful");
            }
        }
        
        Ok(key)
    }


    /// Removes a component from the circuit.
    /// 
    /// This returns [`ReprEditErr::ComponentDoesNotExist`] if the component does not exist.
    pub fn remove_component(&mut self, key: ComponentKey) -> Result<(), ReprEditErr> {
        let props = circ!(self.physical).components.remove(key)
            .ok_or(ReprEditErr::ComponentDoesNotExist)?;

        // Remove from engine (if applicable):
        if let ComponentKey::Function(gate) = key {
            let result = circ!(self.engine).remove_function_node(gate);
            debug_assert!(result, "Engine removal should succeed");
        }
        
        // Handle tunnels specially:
        if matches!(props.inner, PhysicalComponentEnum::Tunnel(_)) {
            if !props.label.is_empty() {
                let sym = circ!(self.physical).tunnel_interner.del_ref(&props.label)
                    .expect("Tunnel should have an assigned symbol");
    
                let result = circ!(self.physical).wires.remove_tunnel(props.origin, sym, &mut circ!(self.engine));
                assert!(result, "Tunnel removal should succeed");
            }
            
        } else {
            // Remove all ports from wire set:
            for index in 0..props.ports.len() {
                let result = circ!(self.physical).wires.remove_port(key, index, &mut circ!(self.engine));
                assert!(result, "Port removal should succeed");
            }
        }

        Ok(())
    }

    /// Adds a wire to the circuit and updates the circuit to properly accommodate the wire.
    /// 
    /// This function handles multiple cases:
    /// - If the new wire endpoint connects to the middle of a wire, the wire creates a junction on the intersecting wire.
    /// - If the new wire overlaps multiple wires, then only wires for the gaps will be created.
    /// 
    /// This raises an error if no wire is added.
    pub fn add_wire(&mut self, w: Wire) -> bool {
        circ!(self.physical).wires.add_wire(w, &mut circ!(self.engine)).is_some()
    }

    /// Removes a wire to the circuit and updates the circuit
    /// to properly accommodate the removed wire.
    /// 
    /// This function removes any wires that overlap the wire range defined by the argument.
    pub fn remove_wire(&mut self, w: Wire) -> bool {
        circ!(self.physical).wires.remove_wire(w, &mut circ!(self.engine))
    }

    /// Checks the component update is valid.
    pub fn check_component_update_ok(&self, components: &[AddComponentArgs<'_>]) -> bool {
        components.iter().all(|&c| self.get_component_bounds(c).is_some())
    }

    /// Moves all items by the specified delta.
    pub fn move_selection(&mut self, components: &[ComponentKey], wires: &[Wire], delta: CoordDelta) -> bool {
        // No movement:
        if delta == (0, 0) {
            return true;
        }

        // Check component bounds are ok:
        let m_new_bounds = components.iter()
            .map(|&k| {
                let component = &circ!(self.physical).components[k];
                let new_origin = coord_add(component.origin, delta)?;
                let bounds = self.get_component_bounds(AddComponentArgs {
                    origin: new_origin,
                    ..component.as_args()
                })?;
                Some((new_origin, bounds))
            })
            .collect::<Option<Vec<_>>>();
        let Some(new_bounds) = m_new_bounds else {
            return false;
        };

        // Check wire bounds are ok:
        let m_new_wires = wires.iter()
            .map(|w| {
                let [p, q] = w.endpoints();
                let np = coord_add(p, delta)?;
                let nq = coord_add(q, delta)?;

                Wire::from_endpoints(np, nq)
            })
            .collect::<Option<Vec<_>>>();
        let Some(new_wires) = m_new_wires else {
            return false;
        };

        // Update all items:
        for (&k, (origin, ComponentBounds { bounds, ports })) in std::iter::zip(components, new_bounds) {
            let component = &mut circ!(self.physical).components[k];
            component.origin = origin;
            component.bounds = bounds;
            let old_ports = std::mem::take(&mut component.ports);
            
            for (i, (old_port, &new_port)) in std::iter::zip(old_ports, &ports).enumerate() {
                let component = &circ!(self.physical).components[k];
                match component.inner {
                    PhysicalComponentEnum::Tunnel(_) => {
                        let sym = circ!(self.physical).tunnel_interner.get(&component.label)
                            .expect("Tunnel should have an assigned symbol");
                        let r = circ!(self.physical).wires.remove_tunnel(old_port, sym, &mut circ!(self.engine));
                        assert!(r, "expected removal to work");
                        
                        circ!(self.physical).wires.add_tunnel(new_port, sym, &mut circ!(self.engine))
                            .expect("addition to work");
                    },
                    _ => {
                        let r = circ!(self.physical).wires.remove_port(k, i, &mut circ!(self.engine));
                        assert!(r, "expected removal to work");

                        circ!(self.physical).wires.add_port(new_port, k, i, &mut circ!(self.engine))
                            .expect("addition to work");
                    }
                }
            }

            circ!(self.physical).components[k].ports = ports;
        }
        for &w in wires {
            self.remove_wire(w);
        }
        for w in new_wires {
            self.add_wire(w);
        }

        true
    }
    
    /// Updates the engine.
    pub fn propagate(&mut self) {
        circ!(self.engine).propagate();
    }

    pub fn get_wire_set(&self) -> &WireSet {
        &circ!(self.physical).wires
    }
    pub fn get_circuit_state(&self) -> &CircuitState {
        &circ!(self.state)
    }
    
    /// Get the component properties for a given component key, returning an error if the component does not exist.
    pub fn get_component(&self, key: ComponentKey) -> Result<&ComponentProps, ReprEditErr> {
        circ!(self.physical).components.get(key).ok_or(ReprEditErr::ComponentDoesNotExist)
    }
    
    /// Checks to see if circuit has a component with the given key
    pub fn has_component(&self, key: ComponentKey) -> bool {
        circ!(self.physical).components.contains_key(key)
    }

    pub fn get_components(&self) -> impl Iterator<Item=(ComponentKey, &ComponentProps)> {
        circ!(self.physical).components.iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::bitarr;
    use crate::middle_end::func::Constant;
    use crate::middle_end::func::Pin;

    use super::*;

    #[test]
    fn middle_repr_connect_wire() {
        let mut repr = MiddleRepr::new();
        let circuit_key = repr.add_circuit("Debug");
        let mut circuit = repr.circuit(circuit_key);

        let [p, q] = [(10, 10), (20, 10)];

        let left = circuit
            .add_component(AddComponentArgs::unlabeled(Pin::new(1, true, Orientation::East), p))
            .unwrap();

        let right = circuit
            .add_component(AddComponentArgs::unlabeled(Pin::new(1, false, Orientation::East), q))
            .unwrap();

        let w = Wire::from_endpoints(p, q).unwrap();
        assert!(circuit.add_wire(w));

        let [lk, rk] = [left, right].map(|key| {
            let ComponentKey::Function(gate) = key else {
                panic!("expected function component");
            };
    
            circuit.get_wire_set()
                .find_key((ComponentKey::Function(gate), 0))
                .unwrap()
        });

        assert_eq!(lk, rk);
    }
    #[test]
    fn middle_repr_connect_wire_not_endpoint() {
        let mut repr = MiddleRepr::new();
        let circuit_key = repr.add_circuit("Debug");
        let mut circuit = repr.circuit(circuit_key);

        let [p, m1, m2, q] = [(10, 10), (15, 10), (25, 10), (30, 10)];
        let left = circuit
            .add_component(AddComponentArgs::unlabeled(Pin::new(1, true, Orientation::East), p))
            .unwrap();

        let right = circuit
            .add_component(AddComponentArgs::unlabeled(Pin::new(1, false, Orientation::East), q))
            .unwrap();

        
        assert!(circuit.add_wire(Wire::from_endpoints(p, m1).unwrap()));
        assert!(circuit.add_wire(Wire::from_endpoints(m2, q).unwrap()));
        assert!(circuit.add_wire(Wire::from_endpoints(m1, m2).unwrap()));

        let [lk, rk] = [left, right].map(|key| {
            let ComponentKey::Function(gate) = key else {
                panic!("expected function component");
            };
    
            circuit.get_wire_set()
                .find_key((ComponentKey::Function(gate), 0))
                .unwrap()
        });

        assert_eq!(lk, rk);
    }

    #[test]
    fn middle_repr_move_component() {
        let mut repr = MiddleRepr::new();
        let circuit_key = repr.add_circuit("Debug");
        let mut circuit = repr.circuit(circuit_key);

        // Add setup:
        let [t0, t1, u0, u1] = [(4, 3), (4, 10), (7, 3), (7, 10)];
        let component = circuit
            .add_component(AddComponentArgs::unlabeled(Constant::new(bitarr![1], Orientation::East), t0))
            .unwrap();

        assert!(circuit.add_wire(Wire::from_endpoints(t0, t1).unwrap()));
        assert!(circuit.add_wire(Wire::from_endpoints(u0, u1).unwrap()));
        
        {
            let wire_set = circuit.get_wire_set();
            let kc = wire_set.find_key((component, 0)).unwrap();
            let kt0 = wire_set.find_key(t0).unwrap();
            let kt1 = wire_set.find_key(t1).unwrap();
            assert_eq!(kc, kt0, "port should have the same value as the first wire");
            assert_eq!(kt0, kt1, "first wire should have the same value throughout");
    
            let ku0 = wire_set.find_key(u0).unwrap();
            let ku1 = wire_set.find_key(u1).unwrap();
            assert_eq!(ku0, ku1);

            circuit.propagate();
            assert_eq!(circuit.get_circuit_state().get_node_value(kt0), bitarr![1]);
            assert_eq!(circuit.get_circuit_state().get_node_value(ku0), bitarr![]);
        }


        // Test move:
        circuit.move_selection(&[component], &[], (3, 0));

        {
            let wire_set = circuit.get_wire_set();
            let kt0 = wire_set.find_key(t0).unwrap();
            let kt1 = wire_set.find_key(t1).unwrap();
            assert_eq!(kt0, kt1);

            let kc = wire_set.find_key((component, 0)).unwrap();
            let ku0 = wire_set.find_key(u0).unwrap();
            let ku1 = wire_set.find_key(u1).unwrap();
            assert_eq!(kc, ku0, "port should have the same value as the second wire");
            assert_eq!(ku0, ku1, "second wire should have the same value throughout");

            circuit.propagate();
            assert_eq!(circuit.get_circuit_state().get_node_value(kt0), bitarr![]);
            assert_eq!(circuit.get_circuit_state().get_node_value(ku0), bitarr![1]);
        }
    }

    #[test]
    fn middle_repr_move_component_wire() {
        let mut repr = MiddleRepr::new();
        let circuit_key = repr.add_circuit("Debug");
        let mut circuit = repr.circuit(circuit_key);

        // Add setup:
        let [sa, t0, t1, sb, u0, u1] = [(3, 3), (4, 3), (4, 10), (6, 3), (7, 3), (7, 10)];
        let component = circuit
            .add_component(AddComponentArgs::unlabeled(Constant::new(bitarr![1], Orientation::East), sa))
            .unwrap();

        let wst = Wire::from_endpoints(sa, t0).unwrap();
        assert!(circuit.add_wire(wst));
        assert!(circuit.add_wire(Wire::from_endpoints(t0, t1).unwrap()));
        assert!(circuit.add_wire(Wire::from_endpoints(u0, u1).unwrap()));
        
        {
            let wire_set = circuit.get_wire_set();
            let kc = wire_set.find_key((component, 0)).unwrap();
            let kst0 = wire_set.find_key(sa).unwrap();
            let kst1 = wire_set.find_key(t0).unwrap();
            let kst2 = wire_set.find_key(t1).unwrap();
            assert_eq!(kc, kst0, "port should have the same value as the first wire");
            assert_eq!(kst0, kst1, "first wire should have the same value throughout");
            assert_eq!(kst1, kst2, "first wire should have the same value throughout");
    
            let ku0 = wire_set.find_key(u0).unwrap();
            let ku1 = wire_set.find_key(u1).unwrap();
            assert_eq!(ku0, ku1);

            circuit.propagate();
            assert_eq!(circuit.get_circuit_state().get_node_value(kst0), bitarr![1]);
            assert_eq!(circuit.get_circuit_state().get_node_value(ku0), bitarr![]);
        }


        // Test move:
        circuit.move_selection(&[component], &[wst], (3, 0));

        {
            let wire_set = circuit.get_wire_set();
            let kt0 = wire_set.find_key(t0).unwrap();
            let kt1 = wire_set.find_key(t1).unwrap();
            assert_eq!(kt0, kt1);

            let kc = wire_set.find_key((component, 0)).unwrap();
            let ksu0 = wire_set.find_key(sb).unwrap();
            let ksu1 = wire_set.find_key(u0).unwrap();
            let ksu2 = wire_set.find_key(u1).unwrap();
            assert_eq!(kc, ksu0, "port should have the same value as the second wire");
            assert_eq!(ksu0, ksu1, "second wire should have the same value throughout");
            assert_eq!(ksu1, ksu2, "second wire should have the same value throughout");

            circuit.propagate();
            assert_eq!(circuit.get_circuit_state().get_node_value(kt0), bitarr![]);
            assert_eq!(circuit.get_circuit_state().get_node_value(ksu0), bitarr![1]);
        }
    }
}