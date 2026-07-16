use std::num::TryFromIntError;
use std::sync::{LazyLock, Mutex};

use anyhow::{Context, Error, anyhow, bail, ensure};
use circuitsim_engine::bitarray::BitArray;
use circuitsim_engine::engine::func::GateKind;
use circuitsim_engine::engine::state::ValueIssue;
use circuitsim_engine::engine::{CircuitKey, FunctionKey, ValueKey};
use circuitsim_engine::middle_end::func::{self, Handedness, Orientation, PhysicalComponentEnum};
use circuitsim_engine::middle_end::wire::Wire;
use circuitsim_engine::middle_end::{ComponentKey, MiddleCircuit, MiddleRepr, UIKey};
use circuitsim_engine::{bitarr, bitstate};
use napi_derive::napi;
use slotmap::KeyData;

static REPR: LazyLock<Mutex<MiddleRepr>> = LazyLock::new(|| Mutex::new(MiddleRepr::new()));

mod js_enum {
    use napi_derive::napi;

    /// Duplicates an enum for use in JS.
    macro_rules! make_napi_enum {
        ($(#[$m:meta])* $vis:vis enum $Js:ident from $Orig:path {
            $($OrigVariant:ident),*
        }) => {
            #[napi(string_enum)]
            $(#[$m])*
            $vis enum $Js {
                $($OrigVariant),*
            }

            impl From<$Orig> for $Js {
                fn from(value: $Orig) -> Self {
                    match value {
                        $(
                            <$Orig>::$OrigVariant => Self::$OrigVariant,
                        )*
                    }
                }
            }
            impl From<$Js> for $Orig {
                fn from(value: $Js) -> Self {
                    match value {
                        $(
                            $Js::$OrigVariant => Self::$OrigVariant,
                        )*
                    }
                }
            }
        }
    }

    make_napi_enum! {
        #[derive(PartialEq, Eq, Debug, Clone, Copy)]
        pub enum Handedness from super::Handedness {
            TopLeft, DownRight
        }
    }
    make_napi_enum! {
        #[derive(PartialEq, Eq, Debug, Clone, Copy)]
        pub enum Orientation from super::Orientation {
            North, South, East, West
        }
    }
}

#[napi(string_enum)]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum KeyKind {
    Circuit,
    Function,
    UI,
    Value,
}
#[napi(object, js_name = "Key")]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct JsKey {
    pub kind: KeyKind,
    pub id: (u32, u32),
    // HACK: Vue's markRaw assigns this field to indicate it shouldn't be reactive.
    #[napi(js_name = "__v_skip")]
    pub __v_skip: bool,
}
impl JsKey {
    fn into_key<K: CastKey>(self) -> anyhow::Result<K> {
        K::try_from_js(self)
    }
}
trait CastKeyByKind: slotmap::Key {
    const KIND: KeyKind;
}
impl<K: CastKeyByKind> CastKey for K {
    fn try_from_js(k: JsKey) -> anyhow::Result<Self> {
        let JsKey {
            kind,
            id,
            __v_skip: _,
        } = k;
        match kind == Self::KIND {
            true => {
                let raw = u64::from(id.0) << 32 | (u64::from(id.1));
                Ok(Self::from(KeyData::from_ffi(raw)))
            }
            false => Err(anyhow!(
                "Expected key of kind {:?}, but got {kind:?}",
                Self::KIND
            )),
        }
    }
    fn into_js(self) -> JsKey {
        let raw = self.data().as_ffi();
        let id = ((raw >> 32) as u32, raw as u32);

        JsKey {
            kind: Self::KIND,
            id,
            __v_skip: true,
        }
    }
}
trait CastKey: Sized {
    fn try_from_js(k: JsKey) -> anyhow::Result<Self>;
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
impl CastKeyByKind for ValueKey {
    const KIND: KeyKind = KeyKind::Value;
}
impl CastKey for ComponentKey {
    fn try_from_js(k: JsKey) -> anyhow::Result<Self> {
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

type Coord = (u32, u32);
#[napi(object)]
pub struct Location {
    pub x: i32,
    pub y: i32,
}
impl From<Coord> for Location {
    fn from(value: Coord) -> Self {
        let (x, y) = value;
        let x = x
            .try_into()
            .expect("bug: repr should bound coordinate to fit in i32");
        let y = y
            .try_into()
            .expect("bug: repr should bound coordinate to fit in i32");
        Self { x, y }
    }
}
impl TryFrom<Location> for (u32, u32) {
    type Error = TryFromIntError;

    fn try_from(value: Location) -> Result<Self, Self::Error> {
        Ok((value.x.try_into()?, value.y.try_into()?))
    }
}

fn get_circuit<'r>(repr: &'r mut MiddleRepr, key: JsKey) -> anyhow::Result<MiddleCircuit<'r>> {
    repr.try_circuit(key.into_key()?)
        .ok_or_else(|| anyhow!("Circuit does not exist"))
}
/// Creates a new circuit and returns its key as an i64 for JS.
#[napi]
pub fn create_circuit(name: String) -> JsKey {
    let mut repr = REPR.lock().unwrap();
    repr.add_circuit(&name).into_js()
}

#[napi]
pub fn add_component(args: CreateComponentArgs) -> anyhow::Result<JsKey> {
    let mut repr = REPR.lock().unwrap();
    
    let component = convert_to_component(&args).unwrap();
   
    let label_orient = args.label_orientation.map_or_else(Default::default, From::from);
    let circuit_key = args.circuit_key.into_key()?;




    let mut circuit = repr.circuit(circuit_key);
    let comp_key = circuit
        .add_component(
            component,
            &args.label.unwrap_or_default(),
            label_orient,
            args.pos.try_into().map_err(|_| anyhow!("Component addition failed"))?,
        )
        .context("Component addition failed")?;
    Ok(comp_key.into_js())
}

 #[napi]
 pub fn validate_placement(args: CreateComponentArgs)-> anyhow::Result<bool>{
    let mut repr = REPR.lock().unwrap();
    
    let component = convert_to_component(&args).unwrap();
   
    let circuit_key = args.circuit_key.into_key()?;
    let circuit = repr.circuit(circuit_key);
    Ok(circuit.validate_placement(component, &args.label.unwrap_or_default(), args.pos.try_into().map_err(|_| anyhow!("Component addition failed"))?))
 }

pub fn convert_to_component(args:&CreateComponentArgs)->Result<PhysicalComponentEnum, Error>{
    let bitsize = args.bitsize.unwrap_or(1);
    let selsize = args.selsize.unwrap_or(1);
    let handedness = args.handedness.map_or_else(Default::default, From::from);
    let orient = args.orientation.map_or_else(Default::default, From::from);
    let bit_array = match &args.constant_value {
        Some(s) => s.parse().context("Could not parse constant value")?,
        None => bitarr![0],
    }
    .resize(bitsize, bitstate![0]);
    let circuit_key = args.circuit_key.into_key()?;
        let inputs = args.inputs.unwrap_or(2);



     let component = match args.component_type.as_str() {
        "PIN" => func::Pin::new(bitsize, args.is_input.unwrap_or(false), orient).into(),
        "CONSTANT" => func::Constant::new(bit_array, orient).into(),
        "SPLITTER" => func::Splitter::new(bitsize, orient, handedness).into(),
        "POWER" => func::Power.into(),
        "GROUND" => func::Ground.into(),
        "TUNNEL" => func::Tunnel::new(orient).into(),
        "PROBE" => func::Probe::new(orient, bitsize).into(),
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
        _ => bail!("Unknown gate type"),
    };
    Ok(component)
}
#[napi]
pub fn remove_component(circuit_key: JsKey, component_key: JsKey) -> anyhow::Result<()> {
    let mut repr = REPR.lock().unwrap();
    let mut circuit = get_circuit(&mut repr, circuit_key)?;
    let key = component_key.into_key()?;

    ensure!(circuit.has_component(key), "Component not found");
    circuit
        .remove_component(key)
        .context("Component removal failed")
}

/// Tries to add the wire to the circuit, returning whether it was successful.
#[napi]
pub fn add_wire(circuit_key: JsKey, start: Location, end: Location) -> anyhow::Result<bool> {
    let mut repr = REPR.lock().unwrap();
    let mut circuit = get_circuit(&mut repr, circuit_key)?;

    let result = if let Ok(start) = start.try_into()
        && let Ok(end) = end.try_into()
        && let Some(w) = Wire::from_endpoints(start, end)
    {
        circuit.add_wire(w).is_ok()
    } else {
        false
    };

    Ok(result)
}
#[napi]

/// Tries to remove the wire from the circuit, returning whether it was successful.
pub fn remove_wire(circuit_key: JsKey, start: Location, end: Location) -> anyhow::Result<bool> {
    let mut repr = REPR.lock().unwrap();
    let mut circuit = get_circuit(&mut repr, circuit_key)?;

    let result = if let Ok(start) = start.try_into()
        && let Ok(end) = end.try_into()
        && let Some(w) = Wire::from_endpoints(start, end)
    {
        circuit.remove_wire(w).is_ok()
    } else {
        false
    };

    Ok(result)
}

#[derive(Debug)]
struct DValueState {
    key: ValueKey,
    value: BitArray,
    issues: Vec<ValueIssue>,
}
impl DValueState {
    fn query(circuit: &MiddleCircuit<'_>, key: ValueKey) -> Self {
        let state = circuit.get_circuit_state();
        let value = state.get_node_value(key);
        let issues = Vec::from_iter(state.get_issues(key).iter().cloned());

        Self { key, value, issues }
    }
}

#[derive(Debug)]
struct DComponentState {
    key: ComponentKey,
    ports: Vec<((u32, u32), Option<DValueState>)>,
    bounds: [(u32, u32); 2],
}

fn get_component_states(circuit: &MiddleCircuit<'_>) -> impl Iterator<Item = DComponentState> {
    circuit.get_components().map(|(ck, props)| {
        let ports = props
            .ports
            .iter()
            .map(|&coord| {
                let value = circuit
                    .get_wire_set()
                    .find_key(coord)
                    .map(|vk| DValueState::query(circuit, vk));
                (coord, value)
            })
            .collect();

        DComponentState {
            key: ck,
            ports,
            bounds: props.bounds,
        }
    })
}

fn get_wire_states(circuit: &MiddleCircuit<'_>) -> impl Iterator<Item = (Wire, DValueState)> {
    circuit
        .get_wire_set()
        .wire_values_iter()
        .map(|(w, k)| (w, DValueState::query(circuit, k)))
}
/// Function Get Transient State, gets the relevant data and state of all components in a circuit

#[napi]
pub fn get_transient_state(
    circuit_key: JsKey,
) -> anyhow::Result<(Vec<TransientComponentState>, Vec<TransientWireState>)> {
    let mut repr = REPR.lock().unwrap();
    let circuit = get_circuit(&mut repr, circuit_key)?;

    let component_states = get_component_states(&circuit)
        .map(|state| {
            let DComponentState {
                key: ckey,
                ports,
                bounds,
            } = state;
            let ports = ports
                .into_iter()
                .map(|(pos, d_value)| {
                    let (vkey, value, issues) = match d_value {
                        Some(DValueState { key, value, issues }) => (
                            Some(key.into_js()),
                            value.to_string(),
                            issues.into_iter().map(|s| s.to_string()).collect(),
                        ),
                        None => (None, String::from("0"), vec![]),
                    };
                    PortTransientState {
                        pos: pos.into(),
                        backend_key: vkey,
                        value,
                        issues,
                    }
                })
                .collect();
            let bounds = bounds.map(Into::into).into();
            TransientComponentState {
                backend_key: ckey.into_js(),
                ports,
                bounds,
            }
        })
        .collect();

    let wire_states = get_wire_states(&circuit)
        .map(|(wire, d_value)| {
            let DValueState { key, value, issues } = d_value;
            let [p, q] = wire.endpoints();
            TransientWireState {
                endpoints: (p.into(), q.into()),
                backend_key: key.into_js(),
                value: value.to_string(),
                issues: issues.into_iter().map(|s| s.to_string()).collect(),
            }
        })
        .collect();

    Ok((component_states, wire_states))
}

#[napi]
pub fn propagate(circuit_key: JsKey) -> anyhow::Result<()> {
    let mut repr = REPR.lock().unwrap();
    let mut circuit = get_circuit(&mut repr, circuit_key)?;

    circuit.propagate();
    Ok(())
}
#[napi]
pub fn print_circuit(circuit_key: JsKey) -> anyhow::Result<String> {
    let mut repr = REPR.lock().unwrap();
    let circuit = get_circuit(&mut repr, circuit_key)?;

    Ok(format!("Circuit:{:?}", circuit))
}

#[napi(object)]
pub struct CreateComponentArgs {
    pub circuit_key: JsKey,
    pub component_type: String, // FIXME: Should be an enum?
    pub label: Option<String>,
    pub label_orientation: Option<js_enum::Orientation>,
    pub pos: Location,

    pub bitsize: Option<u8>,
    pub inputs: Option<u8>,
    pub orientation: Option<js_enum::Orientation>,
    pub constant_value: Option<String>, // FIXME: Should not be String
    pub is_input: Option<bool>,
    pub selsize: Option<u8>,
    pub text_content: Option<String>,
    pub handedness: Option<js_enum::Handedness>,
}
#[napi(object)]
pub struct UpdateComponentArgs {
    pub circuit_key: JsKey,
    pub label: Option<String>,
    pub label_orientation: Option<js_enum::Orientation>,
    pub orientation: Option<js_enum::Orientation>,
    pub text_content: Option<String>,
}
#[napi(object)]
pub struct TransientComponentState {
    pub backend_key: JsKey,
    pub ports: Vec<PortTransientState>,
    pub bounds: (Location, Location),
}
#[napi(object)]
pub struct TransientWireState {
    pub endpoints: (Location, Location),
    pub backend_key: JsKey,
    pub value: String,
    pub issues: Vec<String>,
}
#[napi(object)]
pub struct PortTransientState {
    pub pos: Location,
    pub backend_key: Option<JsKey>,
    pub value: String,
    pub issues: Vec<String>,
}
