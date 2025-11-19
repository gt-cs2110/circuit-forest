
//! Middle-end representation and serialization/deserialization for CircuitSim  files.
//!
//! This module provides a memory representation that pairs the engine `Circuit`
//! (structure + state) with positional/visual metadata (component positions,
//! wire meshes, extras). It also contains helpers to deserialize the legacy
//! `.sim` JSON format (used in `middle_end/latches.sim`) into this memory
//! representation and to serialize an upgraded V2 representation using
//! serde/serde_json.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use slotmap::SecondaryMap;

use crate::circuit::Circuit;
use crate::circuit::graph::{FunctionKey, ValueKey};

/// Positional / visual information for a component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionData {
    pub x: u32,
    pub y: u32,
    /// Raw properties map that came with the legacy format (optional)
    pub properties: Option<HashMap<String, String>>,
}

/// A wire element (mesh) from the legacy format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireMesh {
    pub x: u32,
    pub y: u32,
    pub length: u32,
    pub is_horizontal: bool,
}

/// A single circuit in the middle-end representation.
#[derive(Debug)]
pub struct MiddleCircuit {
    /// The engine circuit (graph + state).
    pub engine: Circuit,

    /// Component positions keyed by engine FunctionKey.
    pub component_pos: SecondaryMap<FunctionKey, PositionData>,

    /// Wire positions keyed by engine ValueKey.
    pub wire_pos: SecondaryMap<ValueKey, Vec<WireMesh>>,

    /// Extra aesthetic nodes (not used yet). Kept as raw JSON values for now.
    pub extras: Vec<serde_json::Value>,
}

impl MiddleCircuit {
    /// Create an empty MiddleCircuit.
    pub fn new() -> Self {
        Self {
            engine: Circuit::new(),
            component_pos: SecondaryMap::new(),
            wire_pos: SecondaryMap::new(),
            extras: Vec::new(),
        }
    }

    /// Add a component to the middle circuit with position data.
    /// Returns the function key of the created component.
    pub fn add_component<F: Into<crate::func::ComponentFn>>(
        &mut self,
        func: F,
        position: PositionData,
    ) -> FunctionKey {
        let key = self.engine.add_function_node(func);
        self.component_pos.insert(key, position);
        key
    }

    /// Remove a component from the middle circuit.
    /// 
    /// Note: This currently only removes position data. Full removal from the
    /// engine graph would require additional implementation in the Circuit type.
    pub fn remove_component(&mut self, key: FunctionKey) -> Option<PositionData> {
        self.component_pos.remove(key)
    }

    /// Add a wire mesh to a value node.
    pub fn add_wire_mesh(&mut self, value: ValueKey, mesh: WireMesh) {
        self.wire_pos
            .entry(value)
            .unwrap()
            .or_default()
            .push(mesh);
    }

    /// Remove all wire meshes associated with a value node.
    pub fn remove_wire_meshes(&mut self, value: ValueKey) -> Option<Vec<WireMesh>> {
        self.wire_pos.remove(value)
    }

    /// Get a reference to the engine circuit.
    pub fn circuit(&self) -> &Circuit {
        &self.engine
    }

    /// Get a mutable reference to the engine circuit.
    pub fn circuit_mut(&mut self) -> &mut Circuit {
        &mut self.engine
    }

    /// Get the position data for a component.
    pub fn get_component_position(&self, key: FunctionKey) -> Option<&PositionData> {
        self.component_pos.get(key)
    }

    /// Update the position of an existing component.
    pub fn update_component_position(&mut self, key: FunctionKey, position: PositionData) -> bool {
        if self.component_pos.contains_key(key) {
            self.component_pos.insert(key, position);
            true
        } else {
            false
        }
    }

    /// Get all component positions as an iterator.
    pub fn component_positions(&self) -> impl Iterator<Item = (FunctionKey, &PositionData)> {
        self.component_pos.iter()
    }

    /// Get the wire meshes for a value node.
    pub fn get_wire_meshes(&self, value: ValueKey) -> Option<&Vec<WireMesh>> {
        self.wire_pos.get(value)
    }

    /// Get all wire positions as an iterator.
    pub fn wire_positions(&self) -> impl Iterator<Item = (ValueKey, &Vec<WireMesh>)> {
        self.wire_pos.iter()
    }

    /// Connect two components by creating a value node and linking it.
    /// Returns the ValueKey of the created wire.
    pub fn connect_components(
        &mut self,
        from_key: FunctionKey,
        from_port: usize,
        to_key: FunctionKey,
        to_port: usize,
    ) -> ValueKey {
        let value = self.engine.add_value_node();
        
        // Connect from output port to wire
        self.engine.connect_one(value, crate::circuit::graph::FunctionPort {
            gate: from_key,
            index: from_port,
        });
        
        // Connect wire to input port
        self.engine.connect_one(value, crate::circuit::graph::FunctionPort {
            gate: to_key,
            index: to_port,
        });
        
        value
    }

    /// Disconnect a wire from all connected ports.
    pub fn disconnect_wire(&mut self, value: ValueKey) {
        // Remove wire meshes
        self.wire_pos.remove(value);
        
        // Note: Disconnecting from engine graph would require additional
        // methods in Circuit type to clear connections
    }

    /// Add an extra (aesthetic) element like a probe or text label.
    pub fn add_extra(&mut self, extra: serde_json::Value) {
        self.extras.push(extra);
    }

    /// Get all extras as a slice.
    pub fn extras(&self) -> &[serde_json::Value] {
        &self.extras
    }

    /// Clear all extras.
    pub fn clear_extras(&mut self) {
        self.extras.clear();
    }

    /// Count the number of components in the circuit.
    pub fn component_count(&self) -> usize {
        self.component_pos.len()
    }

    /// Count the number of wires in the circuit.
    pub fn wire_count(&self) -> usize {
        self.wire_pos.len()
    }

    /// Check if a component exists at the given key.
    pub fn has_component(&self, key: FunctionKey) -> bool {
        self.component_pos.contains_key(key)
    }

    /// Check if a wire exists at the given key.
    pub fn has_wire(&self, key: ValueKey) -> bool {
        self.wire_pos.contains_key(key)
    }

    /// Find components within a rectangular region.
    pub fn find_components_in_region(
        &self,
        min_x: u32,
        min_y: u32,
        max_x: u32,
        max_y: u32,
    ) -> Vec<(FunctionKey, &PositionData)> {
        self.component_pos
            .iter()
            .filter(|(_, pos)| {
                pos.x >= min_x && pos.x <= max_x && pos.y >= min_y && pos.y <= max_y
            })
            .collect()
    }

    /// Move a component by a delta offset.
    pub fn move_component(&mut self, key: FunctionKey, delta_x: i32, delta_y: i32) -> bool {
        if let Some(pos) = self.component_pos.get(key) {
            let new_pos = PositionData {
                x: pos.x.saturating_add_signed(delta_x),
                y: pos.y.saturating_add_signed(delta_y),
                properties: pos.properties.clone(),
            };
            self.component_pos.insert(key, new_pos);
            true
        } else {
            false
        }
    }

    /// Move all components by a delta offset.
    pub fn move_all_components(&mut self, delta_x: i32, delta_y: i32) {
        for (_, pos) in self.component_pos.iter_mut() {
            pos.x = pos.x.saturating_add_signed(delta_x);
            pos.y = pos.y.saturating_add_signed(delta_y);
        }
    }

    /// Clone a component (creates a new component at a different position).
    pub fn clone_component(
        &mut self,
        source_key: FunctionKey,
        new_position: PositionData,
    ) -> Option<FunctionKey> {
        // Get the source component's function type
        let source_func = self.engine.graph().functions.get(source_key)?;
        let func_clone = source_func.func;
        
        // Create new component with the same type
        let new_key = self.engine.add_function_node(func_clone);
        self.component_pos.insert(new_key, new_position);
        
        Some(new_key)
    }

    /// Get a component's property value.
    pub fn get_component_property(&self, key: FunctionKey, property: &str) -> Option<&String> {
        self.component_pos
            .get(key)?
            .properties
            .as_ref()?
            .get(property)
    }

    /// Set a component's property value.
    pub fn set_component_property(
        &mut self,
        key: FunctionKey,
        property: String,
        value: String,
    ) -> bool {
        if let Some(pos) = self.component_pos.get(key) {
            let mut new_pos = pos.clone();
            new_pos
                .properties
                .get_or_insert_with(HashMap::new)
                .insert(property, value);
            self.component_pos.insert(key, new_pos);
            true
        } else {
            false
        }
    }

    /// Remove a component's property.
    pub fn remove_component_property(&mut self, key: FunctionKey, property: &str) -> Option<String> {
        let pos = self.component_pos.get(key)?;
        let mut new_pos = pos.clone();
        let result = new_pos.properties.as_mut()?.remove(property);
        self.component_pos.insert(key, new_pos);
        result
    }
}

/// The top-level middle-end which can hold multiple circuits (a forest).
#[derive(Debug, Default)]
pub struct MiddleEnd {
    /// Map of circuit name to middle circuit.
    pub circuits: HashMap<String, MiddleCircuit>,
}

impl MiddleEnd {
    /// Create a new empty MiddleEnd.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new circuit to the middle-end.
    pub fn add_circuit(&mut self, name: String, circuit: MiddleCircuit) -> Option<MiddleCircuit> {
        self.circuits.insert(name, circuit)
    }

    /// Remove a circuit from the middle-end.
    pub fn remove_circuit(&mut self, name: &str) -> Option<MiddleCircuit> {
        self.circuits.remove(name)
    }

    /// Get a reference to a circuit by name.
    pub fn get_circuit(&self, name: &str) -> Option<&MiddleCircuit> {
        self.circuits.get(name)
    }

    /// Get a mutable reference to a circuit by name.
    pub fn get_circuit_mut(&mut self, name: &str) -> Option<&mut MiddleCircuit> {
        self.circuits.get_mut(name)
    }

    /// Check if a circuit exists.
    pub fn has_circuit(&self, name: &str) -> bool {
        self.circuits.contains_key(name)
    }

    /// Get the number of circuits.
    pub fn circuit_count(&self) -> usize {
        self.circuits.len()
    }

    /// Get all circuit names.
    pub fn circuit_names(&self) -> Vec<&str> {
        self.circuits.keys()
            .map(String::as_str)
            .collect()
    }

    /// Rename a circuit.
    pub fn rename_circuit(&mut self, old_name: &str, new_name: String) -> bool {
        if let Some(circuit) = self.circuits.remove(old_name) {
            self.circuits.insert(new_name, circuit);
            true
        } else {
            false
        }
    }

    /// Clone a circuit with a new name.
    pub fn clone_circuit(&mut self, source_name: &str, new_name: String) -> bool {
        if let Some(source) = self.circuits.get(source_name) {
            // Create a new empty circuit
            let mut new_circuit = MiddleCircuit::new();
            
            // Copy component positions and properties
            for (src_key, pos) in source.component_pos.iter() {
                // Get the function from source
                if let Some(func) = source.engine.graph().functions.get(src_key) {
                    let new_key = new_circuit.engine.add_function_node(func.func);
                    new_circuit.component_pos.insert(new_key, pos.clone());
                }
            }
            
            // Copy wire positions
            for (_src_key, meshes) in source.wire_pos.iter() {
                let new_key = new_circuit.engine.add_value_node();
                new_circuit.wire_pos.insert(new_key, meshes.clone());
            }
            
            // Copy extras
            new_circuit.extras = source.extras.clone();
            
            self.circuits.insert(new_name, new_circuit);
            true
        } else {
            false
        }
    }

    /// Clear all circuits.
    pub fn clear(&mut self) {
        self.circuits.clear();
    }

    /// Get an iterator over all circuits.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &MiddleCircuit)> {
        self.circuits.iter()
    }

    /// Get a mutable iterator over all circuits.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&String, &mut MiddleCircuit)> {
        self.circuits.iter_mut()
    }
}

// -- Legacy schema types (matches provided schema.ts) --

#[derive(Debug, Serialize, Deserialize)]
struct LegacyFile {
    version: String,
    #[serde(rename="globalBitSize")]
    global_bitsize: Option<u8>,
    #[serde(rename="clockSpeed")]
    clock_speed: Option<u64>,
    circuits: Vec<LegacyCircuit>,
    #[serde(rename="revisionSignatures")]
    revision_signatures: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LegacyCircuit {
    name: String,
    components: Vec<LegacyComponent>,
    wires: Vec<LegacyWire>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LegacyComponent {
    name: String,
    x: u32,
    y: u32,
    properties: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LegacyWire {
    x: u32,
    y: u32,
    length: u32,
    #[serde(rename="isHorizontal")]
    is_horizontal: bool,
}

// -- V2 serialization types --

/// Minimal serializable export of a MiddleCircuit (V2).
#[derive(Debug, Serialize, Deserialize)]
pub struct MiddleCircuitV2 {
    pub name: String,
    /// Engine graph exported as lists. Functions and values are given indices
    /// so that connections may be represented by indices as well.
    pub functions: Vec<FunctionSer>,
    pub values: Vec<ValueSer>,
    pub component_pos: Vec<(usize, PositionData)>,
    pub wire_pos: Vec<(usize, Vec<WireMesh>)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionSer {
    pub ty: String,
    pub port_bits: Vec<u8>,
    /// Optionally which value index each port is connected to.
    pub links: Vec<Option<usize>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValueSer {
    pub bitsize: Option<u8>,
    pub links: Vec<(usize, usize)>, // (function_index, port_index)
}

impl MiddleEnd {
    /// Deserialize a legacy `.sim` JSON string into a MiddleEnd.
    ///
    /// This function focuses on creating engine function nodes for known
    /// component names (notably pin peers -> Input/Output) and storing
    /// position metadata. Wire geometry is preserved into `wire_pos` but
    /// wire connections are not inferred (legacy JSON often lacks graph
    /// connectivity information). The goal is a best-effort import so the
    /// frontend can display the components at their positions.
    pub fn from_legacy_json(s: &str) -> Result<Self, serde_json::Error> {
        let lf: LegacyFile = serde_json::from_str(s)?;

        let mut me = MiddleEnd::default();

        for c in lf.circuits {
            let mut mc = MiddleCircuit::new();

            // Keep an index to FunctionKey -> insertion order mapping so we can
            // associate positions to created keys.
            let mut fn_order: Vec<FunctionKey> = Vec::new();
            let mut val_order: Vec<ValueKey> = Vec::new();

            for comp in c.components {
                // For now, only handle PinPeer -> Input/Output mapping. If
                // other component types are needed, extend this match.
                let props = &comp.properties;
                let bitsize = props.get("Bitsize")
                    .and_then(|s| {
                        s.parse::<u8>().ok().or_else(|| {
                            eprintln!(
                                "Warning: Invalid bitsize '{}' for component '{}' at ({}, {}), defaulting to 1",
                                s, comp.name, comp.x, comp.y
                            );
                            None
                        })
                    })
                    .unwrap_or(1u8);

                let _func_key = if comp.name.contains("wiring.PinPeer") {
                    // direction and "Is input?" in properties determine type
                    match props.get("Is input?").map(|s| s.as_str()) {
                        Some("Yes") => {
                            // Create Input with correct bitsize from the start
                            let f = mc.engine.add_function_node(crate::func::Input::new(bitsize));
                            let v = mc.engine.add_value_node();
                            mc.engine.connect_all(f, &[v]);
                            fn_order.push(f);
                            val_order.push(v);
                            mc.component_pos.insert(f, PositionData { 
                                x: comp.x, 
                                y: comp.y, 
                                properties: Some(props.clone()) 
                            });
                            f
                        }
                        Some("No") => {
                            // Output
                            let f = mc.engine.add_function_node(crate::func::Output::new(bitsize));
                            let v = mc.engine.add_value_node();
                            mc.engine.connect_all(f, &[v]);
                            fn_order.push(f);
                            val_order.push(v);
                            mc.component_pos.insert(f, PositionData { 
                                x: comp.x, 
                                y: comp.y, 
                                properties: Some(props.clone()) 
                            });
                            f
                        }
                        _ => {
                            // Default to Output
                            let f = mc.engine.add_function_node(crate::func::Output::new(bitsize));
                            let v = mc.engine.add_value_node();
                            mc.engine.connect_all(f, &[v]);
                            fn_order.push(f);
                            val_order.push(v);
                            mc.component_pos.insert(f, PositionData { 
                                x: comp.x, 
                                y: comp.y, 
                                properties: Some(props.clone()) 
                            });
                            f
                        }
                    }
                } else {
                    // Unknown component: add a generic Output node with bitsize
                    eprintln!("Warning: Unknown component type '{}' at ({}, {}), creating Output", 
                              comp.name, comp.x, comp.y);
                    let f = mc.engine.add_function_node(crate::func::Output::new(bitsize));
                    let v = mc.engine.add_value_node();
                    mc.engine.connect_all(f, &[v]);
                    fn_order.push(f);
                    val_order.push(v);
                    mc.component_pos.insert(f, PositionData { 
                        x: comp.x, 
                        y: comp.y, 
                        properties: Some(props.clone()) 
                    });
                    f
                };
            }

            // Store wires geometry (legacy wires). We can't map them to engine
            // ValueKey indices deterministically because the legacy format
            // doesn't specify which value each mesh belongs to. We therefore
            // keep them as extras under the circuit-level entries.
            for w in c.wires {
                // push to extras to keep the raw geometry available
                mc.extras.push(serde_json::to_value(&w).unwrap_or(serde_json::Value::Null));
            }

            me.circuits.insert(c.name, mc);
        }

        Ok(me)
    }

    /// Export the middle-end into a V2 JSON string.
    /// This performs a best-effort export of engine graphs and position maps.
    pub fn to_v2_json(&self) -> Result<String, serde_json::Error> {
        let mut circuits_v2: Vec<MiddleCircuitV2> = Vec::new();

        for (name, mc) in &self.circuits {
            // Build function and value index maps so we can reference indices
            // consistently in the serialized output.
            let mut fn_index_map = HashMap::new();
            let mut fns: Vec<FunctionSer> = Vec::new();

            for (i, (k, f)) in mc.engine.graph().functions.iter().enumerate() {
                fn_index_map.insert(k, i);
                // get port bit widths
                let port_bits = f.port_props.iter().map(|p| p.bitsize).collect();
                // links: convert Option<ValueKey> to Option<usize> by looking up value index later
                // temporarily store None; we'll fill after values are enumerated
                let links = f.links.iter().map(|opt| opt.map(|_| 0usize)).collect();
                // Use a cleaner type name instead of Debug format
                let ty = match &f.func {
                    crate::func::ComponentFn::And(_) => "And",
                    crate::func::ComponentFn::Or(_) => "Or",
                    crate::func::ComponentFn::Xor(_) => "Xor",
                    crate::func::ComponentFn::Nand(_) => "Nand",
                    crate::func::ComponentFn::Nor(_) => "Nor",
                    crate::func::ComponentFn::Xnor(_) => "Xnor",
                    crate::func::ComponentFn::Not(_) => "Not",
                    crate::func::ComponentFn::TriState(_) => "TriState",
                    crate::func::ComponentFn::Input(_) => "Input",
                    crate::func::ComponentFn::Output(_) => "Output",
                    crate::func::ComponentFn::Constant(_) => "Constant",
                    crate::func::ComponentFn::Splitter(_) => "Splitter",
                    crate::func::ComponentFn::Mux(_) => "Mux",
                    crate::func::ComponentFn::Demux(_) => "Demux",
                    crate::func::ComponentFn::Decoder(_) => "Decoder",
                    crate::func::ComponentFn::Register(_) => "Register",
                }.to_string();
                fns.push(FunctionSer { ty, port_bits, links });
            }

            let mut val_index_map = HashMap::new();
            let mut vals: Vec<ValueSer> = Vec::new();
            for (i, (k, v)) in mc.engine.graph().values.iter().enumerate() {
                val_index_map.insert(k, i);
                // Build a list of all links (function_index, port_index)
                let links: Vec<(usize, usize)> = v.links.iter().map(|port| {
                    let fn_i = *fn_index_map.get(&port.gate).expect("function key missing");
                    (fn_i, port.index)
                }).collect();
                vals.push(ValueSer { bitsize: v.bitsize, links });
            }

            // Now fix function link indices to reference value indices
            for (fi, (_k, f)) in mc.engine.graph().functions.iter().enumerate() {
                let mut new_links = Vec::with_capacity(f.links.len());
                for opt in &f.links {
                    if let Some(vk) = opt {
                        let v_i = *val_index_map.get(vk).expect("value key missing");
                        new_links.push(Some(v_i));
                    } else {
                        new_links.push(None);
                    }
                }
                fns[fi].links = new_links;
            }

            // component_pos: convert SecondaryMap(FunctionKey -> PositionData) to (function_index, PositionData)
            let mut component_pos: Vec<(usize, PositionData)> = Vec::new();
            for (k, p) in mc.component_pos.iter() {
                if let Some(&fi) = fn_index_map.get(&k) {
                    component_pos.push((fi, p.clone()));
                }
            }

            // wire_pos: ValueKey -> Vec<WireMesh>
            let mut wire_pos: Vec<(usize, Vec<WireMesh>)> = Vec::new();
            for (k, meshes) in mc.wire_pos.iter() {
                if let Some(&vi) = val_index_map.get(&k) {
                    wire_pos.push((vi, meshes.clone()));
                }
            }

            circuits_v2.push(MiddleCircuitV2 {
                name: name.clone(),
                functions: fns,
                values: vals,
                component_pos,
                wire_pos,
            });
        }

        serde_json::to_string_pretty(&circuits_v2)
    }

    /// Convenience: load legacy `.sim` from file path and return MiddleEnd.
    pub fn from_legacy_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let s = std::fs::read_to_string(path)?;
        Ok(Self::from_legacy_json(&s)?)
    }

    /// Convenience: write V2 JSON to path (pretty-printed).
    pub fn write_v2_file(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let s = self.to_v2_json()?;
        std::fs::write(path, s)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests_serialization;

#[cfg(test)]
mod test_operations;

