use std::sync::{LazyLock, Mutex};

use circuitsim_engine::bitarr;
use circuitsim_engine::engine::FunctionKey;
use circuitsim_engine::engine::func::GateKind;
use circuitsim_engine::middle_end::func::{self, Handedness, Orientation, PhysicalComponentEnum};
use circuitsim_engine::middle_end::wire::Wire;
use circuitsim_engine::middle_end::{ComponentKey, MiddleRepr, UIKey};
use napi::bindgen_prelude::BigInt;
use napi_derive::napi;
use slotmap::KeyData;

static REPR: LazyLock<Mutex<MiddleRepr>> = LazyLock::new(|| Mutex::new(MiddleRepr::new()));

/// Creates a new circuit and returns its key as an i64 for JS.
#[napi]
pub fn create_circuit(name: String) -> Result<BigInt, napi::Error> {
    let mut repr = REPR.lock().unwrap();
    let key = repr.add_circuit(&name);
    Ok(key_to_bigint(key))
}

#[napi]
pub fn add_component(args: CreateComponentArgs) -> Result<BigInt, napi::Error> {
    let mut rep = REPR.lock().unwrap();
    let bitsize = args.bitsize.unwrap_or(1);
    let selsize = args.selsize.unwrap_or(1);
    let orient = match args.orientation {
        Some(i) => Orientation::try_from(i)
            .map_err(|_| napi::Error::from_reason("Invalid orientation value"))?,
        None => Default::default(),
    };
    let label_orient = match args.label_orientation {
        Some(i) => Orientation::try_from(i)
            .map_err(|_| napi::Error::from_reason("Invalid label orientation value"))?,
        None => Default::default(),
    };
    let handedness = match args.handedness {
        Some(i) => Handedness::try_from(i)
            .map_err(|_| napi::Error::from_reason("Invalid handedness value"))?,
        None => Default::default(),
    };
    let bit_array = match args.constant_value {
        Some(s) => s
            .parse()
            .map_err(|_| napi::Error::from_reason("Invalid constant value"))?,
        None => bitarr![0],
    };

    let inputs = args.inputs.unwrap_or(2);
    let component: PhysicalComponentEnum = match args.component_type.as_str() {
        "PIN" => func::Pin::new(bitsize, args.is_input.unwrap_or(false), orient).into(),
        "CONSTANT" => func::Constant::new(bit_array, orient).into(),
        "SPLITTER" => func::Splitter::new(bitsize, orient, handedness).into(),
        "POWER" => func::Power.into(),
        "GROUND" => func::Ground.into(),
        "TUNNEL" => func::Tunnel::new(orient).into(),
        "PROBE" => func::Probe::new(orient).into(),
        "MUX" => func::Mux::new(bitsize, selsize, orient, handedness).into(),
        "DEMUX" => func::Demux::new(bitsize, selsize, orient, handedness).into(),
        "DECODER" => func::Decoder::new(selsize, orient, handedness).into(),
        "TEXT" => func::Text.into(),
        "SUBCIRCUIT" => func::Subcircuit::new(
            bigint_to_key(&args.circuit_key)
                .ok_or_else(|| napi::Error::from_reason("Invalid circuit key"))?,
        )
        .into(),
        "NOT" => func::Not::new(bitsize, orient).into(),
        "BUFFER" => func::TriState::new(bitsize, orient, handedness).into(),
        "AND" => func::Gate::new(GateKind::And, bitsize, inputs, orient).into(),
        "OR" => func::Gate::new(GateKind::Or, bitsize, inputs, orient).into(),
        "NAND" => func::Gate::new(GateKind::Nand, bitsize, inputs, orient).into(),
        "NOR" => func::Gate::new(GateKind::Nor, bitsize, inputs, orient).into(),
        "XOR" => func::Gate::new(GateKind::Xor, bitsize, inputs, orient).into(),
        "XNOR" => func::Gate::new(GateKind::Xnor, bitsize, inputs, orient).into(),
        _ => return Err(napi::Error::from_reason("Unknown gate type")),
    };

    let mut circuit = rep.circuit(
        bigint_to_key(&args.circuit_key)
            .ok_or_else(|| napi::Error::from_reason("Invalid circuit key"))?,
    );

    let cmpkey = circuit
        .add_component(
            component,
            &args.label.unwrap_or_default(),
            label_orient,
            (args.x, args.y),
        )
        .map_err(|_| napi::Error::from_reason("Component edit failed"))?;
    let big = match cmpkey {
        //unwrap the component key into either type and convert to bigint
        ComponentKey::Function(k) => key_to_bigint(k),
        ComponentKey::UI(k) => key_to_bigint(k),
    };
    Ok(big)
}
#[napi]
pub fn remove_component(circuit_key: BigInt, component_key: BigInt) -> Result<(), napi::Error> {
    let mut rep = REPR.lock().unwrap();
    let key = bigint_to_key(&circuit_key)
        .ok_or_else(|| napi::Error::from_reason("invalid circuit key"))?;
    if !rep.has_circuit(key) {
        return Err(napi::Error::from_reason("Circuit not found"));
    }
    let mut circuit = rep.circuit(key);

    if let Some(fk) = bigint_to_key::<FunctionKey>(&component_key) {
        if !circuit.has_component(ComponentKey::Function(fk)) {
            return Err(napi::Error::from_reason("Component not found"));
        }
        if circuit.remove_component(ComponentKey::Function(fk)).is_ok() {
            return Ok(());
        }
    }

    if let Some(uk) = bigint_to_key::<UIKey>(&component_key) {
        if !circuit.has_component(ComponentKey::UI(uk)) {
            return Err(napi::Error::from_reason("Component not found"));
        }
        return circuit
            .remove_component(ComponentKey::UI(uk))
            .map_err(|_| napi::Error::from_reason("Component removal failed"));
    }

    Err(napi::Error::from_reason("Invalid component key"))
}
#[napi]

pub fn add_wire(circuit_key: BigInt, wire: TransientWireState) -> Result<(), napi::Error> {
    let mut rep = REPR.lock().unwrap();
    let key = bigint_to_key(&circuit_key).unwrap();
    if !rep.has_circuit(key) {
        return Err(napi::Error::from_reason("Circuit not found"));
    };

    let mut circuit = rep.circuit(key);
    //coordinates are loermost  x and y so smallest value
    let x = std::cmp::min(wire.endpoints[0].x, wire.endpoints[1].x);
    let y = std::cmp::min(wire.endpoints[0].y, wire.endpoints[1].y);
    // circuit.add_wire(Wire{x, y, length:NonZero::new(wire.length).unwrap(), horizontal:wire.isHorizantal}).map_err(|op| napi::Error::from_reason(op.to_string()))
    let res = circuit.add_wire(Wire::new(x, y, wire.length, wire.is_horizontal).unwrap());
    if let Err(err) = res {
        println!("Error creating wire: {}", err);
        return Err(napi::Error::from_reason(err.to_string()));
    }
    Ok(())
}

/// Function Get Transient State, gets the relevant data and state of all components in a circuit

#[napi]
pub fn get_transient_state(
    circuit_key: BigInt,
) -> Result<(Vec<TransientComponentState>, Vec<TransientWireState>), napi::Error> {
    let mut component_states: Vec<TransientComponentState> = Vec::new();
    let mut rep = REPR.lock().unwrap();
    let key = bigint_to_key(&circuit_key)
        .ok_or_else(|| napi::Error::from_reason("invalid circuit key"))?;
    if !rep.has_circuit(key) {
        return Err(napi::Error::from_reason("Circuit not found"));
    }
    let circuit = rep.circuit(key);

    for (key, state) in circuit.get_component_states() {
        let big_int = key_to_bigint(key);
        let component = circuit
            .get_component(ComponentKey::Function(key))
            .map_err(|_| napi::Error::from_reason("Component not found"))?;
        //get num ports and iterate through them to get values and states
        let num_ports = state.get_num_ports();
        let ports: Vec<PortTransientState> = (0..num_ports)
            .map(|i| {
                let (x, y) = component.ports[i];
                let value = state.get_port(i).to_string();
                let value_key: circuitsim_engine::engine::ValueKey = circuit
                    .get_wire_set()
                    .find_key((ComponentKey::Function(key), i))
                    .unwrap();
                let issues = circuit
                    .get_circuit_state()
                    .get_issues(value_key)
                    .iter()
                    .map(|issue| issue.to_string())
                    .collect();

                PortTransientState {
                    x,
                    y,
                    value,
                    issues,
                }
            })
            .collect();
        //check to see if component is a probe or constant to get value for component value field
        let component_value = match component.inner {
            PhysicalComponentEnum::Probe(probe) => Some("0".to_string()), // Need to figure out how to get the actual value of the probe
            PhysicalComponentEnum::Constant(constant) => Some(constant.get_value().to_string()),
            _ => None,
        };
        component_states.push(TransientComponentState {
            backend_key: big_int.get_i128().0.to_string(),
            ports,
            bounds: vec![
                Location {
                    x: component.bounds[0].0,
                    y: component.bounds[0].1,
                },
                Location {
                    x: component.bounds[1].0,
                    y: component.bounds[1].1,
                },
            ],
            component_value,
        });
    }

    //Get Wire Transient States
    //Middle End has a wire set and tunnel interner'

    //Wire Range Map holds all our horizantal and vertical segments

    let mut wire_states: Vec<TransientWireState> = Vec::new();

    for (wire, value, issues) in circuit.get_wire_states() {
        let [(lx, ly), (rx, ry)] = wire.endpoints();
        wire_states.push(TransientWireState {
            endpoints: vec![Location { x: lx, y: ly }, Location { x: rx, y: ry }],
            is_horizontal: wire.horizontal(),
            length: wire.length(),
            value: value.to_string(),
            issues: issues.into_iter().map(|s| s.to_string()).collect(),
        });
    }

    Ok((component_states, wire_states))
}

#[napi]
pub fn propagate(circuit_key: BigInt) -> Result<(), napi::Error> {
    let mut rep = REPR.lock().unwrap();
    let key = bigint_to_key(&circuit_key)
        .ok_or_else(|| napi::Error::from_reason("invalid circuit key"))?;
    if !rep.has_circuit(key) {
        return Err(napi::Error::from_reason("Circuit not found"));
    }

    let mut circuit = rep.circuit(key);
    circuit.propagate();
    Ok(())
}
#[napi]
pub fn print_circuit(circuit_key: BigInt) -> Result<String, napi::Error> {
    let mut rep = REPR.lock().unwrap();
    let key = bigint_to_key(&circuit_key)
        .ok_or_else(|| napi::Error::from_reason("invalid circuit key"))?;
    if !rep.has_circuit(key) {
        return Err(napi::Error::from_reason("Circuit not found"));
    }
    let circuit = rep.circuit(key);
    Ok(format!("Circuit:{:?}", circuit))
}

pub fn key_to_bigint<K: slotmap::Key>(k: K) -> BigInt {
    let raw = k.data().as_ffi(); // u64
    BigInt::from(raw)
}

fn bigint_to_key<K: slotmap::Key>(b: &BigInt) -> Option<K> {
    let (sign, raw, lossless) = b.get_u64();
    if sign || !lossless {
        return None;
    }

    Some(K::from(KeyData::from_ffi(raw)))
}

#[napi(object)]
pub struct CreateComponentArgs {
    pub circuit_key: BigInt,
    pub component_type: String,
    pub bitsize: Option<u8>,
    pub inputs: Option<u8>,
    pub orientation: Option<u8>,
    pub label: Option<String>,
    pub x: u32,
    pub y: u32,
    pub label_orientation: Option<u8>,
    pub constant_value: Option<String>,
    pub is_input: Option<bool>,
    pub selsize: Option<u8>,
    pub text_content: Option<String>,
    pub handedness: Option<u8>,
}
#[napi(object)]
pub struct TransientComponentState {
    pub backend_key: String,
    pub ports: Vec<PortTransientState>,
    pub bounds: Vec<Location>,
    pub component_value: Option<String>, //only for probes and constants
}
#[napi(object)]
pub struct TransientWireState {
    pub endpoints: Vec<Location>,
    pub is_horizontal: bool,
    pub length: u32,
    pub value: String,
    pub issues: Vec<String>,
}
#[napi(object)]
pub struct PortTransientState {
    pub x: u32,
    pub y: u32,
    pub value: String,
    pub issues: Vec<String>,
}
#[napi(object)]
pub struct Location {
    pub x: u32,
    pub y: u32,
}
