use std::sync::{LazyLock, Mutex};

use circuitsim_engine::bitarr;
use circuitsim_engine::engine::func::GateKind;
use circuitsim_engine::engine::{CircuitKey, FunctionKey};
use circuitsim_engine::middle_end::func::{self, Handedness, Orientation, PhysicalComponentEnum};
use circuitsim_engine::middle_end::wire::Wire;
use circuitsim_engine::middle_end::{ComponentKey, MiddleCircuit, MiddleRepr, UIKey};
use napi_derive::napi;
use slotmap::KeyData;

static REPR: LazyLock<Mutex<MiddleRepr>> = LazyLock::new(|| Mutex::new(MiddleRepr::new()));

#[napi(string_enum)]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum KeyKind {
    Circuit,
    Function,
    UI,
}
#[napi(object, js_name = "Key")]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct JsKey {
    pub kind: KeyKind,
    pub id: (u32, u32),
}
impl JsKey {
    fn into_key<K: CastKey>(self) -> napi::Result<K> {
        K::try_from_js(self)
    }
}
trait CastKeyByKind: slotmap::Key {
    const KIND: KeyKind;
}
impl<K: CastKeyByKind> CastKey for K {
    fn try_from_js(k: JsKey) -> napi::Result<Self> {
        let JsKey { kind, id } = k;
        match kind == Self::KIND {
            true => {
                let raw = u64::from(id.0) << 32 | (u64::from(id.1));
                Ok(Self::from(KeyData::from_ffi(raw)))
            }
            false => Err(napi::Error::from_reason(format!(
                "Expected key of kind {:?}, but got {kind:?}",
                Self::KIND
            ))),
        }
    }
    fn into_js(self) -> JsKey {
        let raw = self.data().as_ffi();
        let id = ((raw >> 32) as u32, raw as u32);

        JsKey {
            kind: Self::KIND,
            id,
        }
    }
}
trait CastKey: Sized {
    fn try_from_js(k: JsKey) -> napi::Result<Self>;
    fn into_js(self) -> JsKey;
}
impl CastKeyByKind for CircuitKey {
    const KIND: KeyKind = KeyKind::Circuit;
}
impl CastKeyByKind for FunctionKey {
    const KIND: KeyKind = KeyKind::Function;
}
impl CastKeyByKind for UIKey {
    const KIND: KeyKind = KeyKind::UI;
}
impl CastKey for ComponentKey {
    fn try_from_js(k: JsKey) -> napi::Result<Self> {
        Ok(match k.kind == KeyKind::UI {
            true => UIKey::try_from_js(k)?.into(),
            false => FunctionKey::try_from_js(k)?.into(),
        })
    }

    fn into_js(self) -> JsKey {
        match self {
            ComponentKey::Function(k) => k.into_js(),
            ComponentKey::UI(k) => k.into_js(),
        }
    }
}

#[napi(object)]
pub struct Location {
    pub x: u32,
    pub y: u32,
}
impl From<(u32, u32)> for Location {
    fn from(value: (u32, u32)) -> Self {
        let (x, y) = value;
        Self { x, y }
    }
}
impl From<Location> for (u32, u32) {
    fn from(value: Location) -> Self {
        (value.x, value.y)
    }
}

fn get_circuit<'r>(repr: &'r mut MiddleRepr, key: JsKey) -> Result<MiddleCircuit<'r>, napi::Error> {
    repr.try_circuit(key.into_key()?)
        .ok_or_else(|| napi::Error::from_reason("circuit does not exist"))
}
/// Creates a new circuit and returns its key as an i64 for JS.
#[napi]
pub fn create_circuit(name: String) -> JsKey {
    let mut repr = REPR.lock().unwrap();
    repr.add_circuit(&name).into_js()
}

#[napi]
pub fn add_component(args: CreateComponentArgs) -> Result<JsKey, napi::Error> {
    let mut repr = REPR.lock().unwrap();
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
    let circuit_key = args.circuit_key.into_key()?;

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
        "SUBCIRCUIT" => func::Subcircuit::new(circuit_key).into(), // TODO: I don't believe this is correct
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

    let mut circuit = repr.circuit(circuit_key);

    let comp_key = circuit
        .add_component(
            component,
            &args.label.unwrap_or_default(),
            label_orient,
            (args.x, args.y),
        )
        .map_err(|_| napi::Error::from_reason("Component edit failed"))?;
    Ok(comp_key.into_js())
}
#[napi]
pub fn remove_component(circuit_key: JsKey, component_key: JsKey) -> Result<(), napi::Error> {
    let mut repr = REPR.lock().unwrap();
    let mut circuit = get_circuit(&mut repr, circuit_key)?;
    let key = component_key.into_key()?;
    if !circuit.has_component(key) {
        return Err(napi::Error::from_reason("Component not found"));
    }

    circuit
        .remove_component(key)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

#[napi]
pub fn add_wire(circuit_key: JsKey, wire: TransientWireState) -> Result<(), napi::Error> {
    let mut repr = REPR.lock().unwrap();
    let mut circuit = get_circuit(&mut repr, circuit_key)?;

    let (p, q) = wire.endpoints;
    let w = Wire::from_endpoints(p.into(), q.into()).ok_or_else(|| {
        napi::Error::from_reason("Could not construct wire: Wire is not straight")
    })?;

    circuit
        .add_wire(w)
        .map_err(|e| napi::Error::from_reason(format!("Could not construct wire: {e}")))
}

/// Function Get Transient State, gets the relevant data and state of all components in a circuit

#[napi]
pub fn get_transient_state(
    circuit_key: JsKey,
) -> Result<(Vec<TransientComponentState>, Vec<TransientWireState>), napi::Error> {
    let mut component_states: Vec<TransientComponentState> = Vec::new();
    let mut repr = REPR.lock().unwrap();
    let circuit = get_circuit(&mut repr, circuit_key)?;

    for (key, state) in circuit.get_component_states() {
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

        let bounds = component.bounds.map(Into::into).into();
        // Get value for component value field for Probe or Constant
        let bitvalue = match component.inner {
            PhysicalComponentEnum::Probe(_) => Some(state.get_port(0)),
            PhysicalComponentEnum::Constant(constant) => Some(constant.get_value()),
            _ => None,
        };

        component_states.push(TransientComponentState {
            backend_key: key.into_js(),
            ports,
            bounds,
            component_value: bitvalue.map(|s| s.to_string()),
        });
    }

    //Get Wire Transient States
    //Middle End has a wire set and tunnel interner'

    //Wire Range Map holds all our horizantal and vertical segments

    let mut wire_states: Vec<TransientWireState> = Vec::new();

    for (wire, value, issues) in circuit.get_wire_states() {
        let [p, q] = wire.endpoints();
        wire_states.push(TransientWireState {
            endpoints: (p.into(), q.into()),
            is_horizontal: wire.horizontal(),
            length: wire.length(),
            value: value.to_string(),
            issues: issues.into_iter().map(|s| s.to_string()).collect(),
        });
    }

    Ok((component_states, wire_states))
}

#[napi]
pub fn propagate(circuit_key: JsKey) -> Result<(), napi::Error> {
    let mut repr = REPR.lock().unwrap();
    let mut circuit = get_circuit(&mut repr, circuit_key)?;

    circuit.propagate();
    Ok(())
}
#[napi]
pub fn print_circuit(circuit_key: JsKey) -> Result<String, napi::Error> {
    let mut repr = REPR.lock().unwrap();
    let circuit = get_circuit(&mut repr, circuit_key)?;

    Ok(format!("Circuit:{:?}", circuit))
}

#[napi(object)]
pub struct CreateComponentArgs {
    pub circuit_key: JsKey,
    pub component_type: String,
    pub label: Option<String>,
    pub label_orientation: Option<u8>,
    pub x: u32,
    pub y: u32,

    pub bitsize: Option<u8>,
    pub inputs: Option<u8>,
    pub orientation: Option<u8>,
    pub constant_value: Option<String>,
    pub is_input: Option<bool>,
    pub selsize: Option<u8>,
    pub text_content: Option<String>,
    pub handedness: Option<u8>,
}
#[napi(object)]
pub struct TransientComponentState {
    pub backend_key: JsKey,
    pub ports: Vec<PortTransientState>,
    pub bounds: (Location, Location),
    pub component_value: Option<String>, //only for probes and constants
}
#[napi(object)]
pub struct TransientWireState {
    pub endpoints: (Location, Location),
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
