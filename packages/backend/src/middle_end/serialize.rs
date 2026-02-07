use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::middle_end::Wire;

#[derive(Serialize, Deserialize, Debug)]
struct CircuitFile {
    /// CircuitSim version
    version: String,

    /// Global bit size (1-32)
    #[serde(rename = "globalBitSize")]
    global_bitsize: u32,
    
    /// Clock speed
    #[serde(rename = "clockSpeed")]
    clock_speed: u32,

    /// All defined circuits in this file.
    circuits: Vec<CircuitInfo>,

    /// A set of hashes which keeps track of all updates to the file.
    #[serde(rename = "revisionSignatures")]
    revision_signatures: Vec<String>
}

#[derive(Serialize, Deserialize, Debug)]
struct CircuitInfo {
    /// Name of the circuit.
    name: String,

    /// Components in the circuit.
    components: Vec<ComponentInfo>,

    /// Wires in circuit.
    wires: Vec<Wire>
}

#[derive(Serialize, Deserialize, Debug)]
struct ComponentInfo {
    /// Component type
    name: String,
    x: u32,
    y: u32,
    properties: HashMap<String, ()>
}
