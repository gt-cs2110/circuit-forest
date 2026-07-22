use std::num::TryFromIntError;
use std::sync::{LazyLock, Mutex};

use anyhow::{Context, anyhow, bail, ensure};
use circuitsim_engine::bitarray::BitArray;
use circuitsim_engine::engine::func::GateKind;
use circuitsim_engine::engine::state::ValueIssue;
use circuitsim_engine::engine::{CircuitKey, ValueKey};
use circuitsim_engine::middle_end::func::{self, Handedness, Orientation, PhysicalComponentEnum};
use circuitsim_engine::middle_end::wire::Wire;
use circuitsim_engine::middle_end::{AddComponentArgs, ComponentKey, MiddleCircuit, MiddleRepr};
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
    Component,
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
impl CastKeyByKind for ValueKey {
    const KIND: KeyKind = KeyKind::Value;
}
impl CastKeyByKind for ComponentKey {
    const KIND: KeyKind = KeyKind::Component;
}

type Coord = (u32, u32);
#[napi(object)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
pub fn add_component(circuit_key: JsKey, args: CreateComponentArgs) -> anyhow::Result<JsKey> {
    let mut repr = REPR.lock().unwrap();
    let mut circuit = get_circuit(&mut repr, circuit_key)?;

    let args = AddComponentArgs::try_from(&args)?;
    let comp_key = circuit
        .add_component(args)
        .context("Component addition failed")?;
    Ok(comp_key.into_js())
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
        circuit.add_wire(w)
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
        circuit.remove_wire(w)
    } else {
        false
    };

    Ok(result)
}

/// Tries to update all of the components specified in arguments.
///
/// This returns whether the update succeeded.
#[napi]
pub fn update_components(
    circuit_key: JsKey,
    args: Vec<(JsKey, CreateComponentArgs)>,
) -> anyhow::Result<bool> {
    let mut repr = REPR.lock().unwrap();
    let mut circuit = get_circuit(&mut repr, circuit_key)?;

    let items: Vec<_> = args
        .iter()
        .map(|(k, args)| {
            Ok((
                k.into_key::<ComponentKey>()?,
                AddComponentArgs::try_from(args)?,
            ))
        })
        .collect::<anyhow::Result<_>>()?;

    let result = circuit.batch_construct_and_overwrite(items, []);
    Ok(result)
}

#[napi]
pub fn move_selection(
    circuit_key: JsKey,
    components: Vec<JsKey>,
    wires: Vec<(Location, Location)>,
    delta: Location,
) -> anyhow::Result<bool> {
    let mut repr = REPR.lock().unwrap();
    let mut circuit = get_circuit(&mut repr, circuit_key)?;

    let components: Vec<_> = components
        .into_iter()
        .map(|k| k.into_key())
        .collect::<Result<_, _>>()?;

    let wires: Vec<_> = wires
        .into_iter()
        .map(|(p, q)| {
            let p = p.try_into()?;
            let q = q.try_into()?;
            Wire::from_endpoints(p, q).ok_or_else(|| anyhow!("{p:?}, {q:?} does not form wire"))
        })
        .collect::<Result<_, _>>()?;

    let Location { x, y } = delta;
    let result = circuit.batch_move(&components, &wires, (x, y));
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
    origin: (u32, u32),
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
            origin: props.origin,
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
                origin,
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
                pos: origin.into(),
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
    pub port_assignments: Option<Vec<Option<u8>>>,
    pub num_legs: Option<u8>,
}
impl<'a> TryFrom<&'a CreateComponentArgs> for AddComponentArgs<'a> {
    type Error = anyhow::Error;

    fn try_from(args: &'a CreateComponentArgs) -> Result<Self, Self::Error> {
        let bitsize = args
            .bitsize
            .unwrap_or(if args.component_type == "SPLITTER" {
                2
            } else {
                1
            });
        let selsize = args.selsize.unwrap_or(1);
        let handedness = args.handedness.map_or_else(
            || {
                if args.component_type == "SPLITTER" {
                    Handedness::TopLeft
                } else {
                    Default::default()
                }
            },
            From::from,
        );
        let orient = args.orientation.map_or_else(Default::default, From::from);
        let bit_array = match &args.constant_value {
            Some(s) => s.parse().context("Could not parse constant value")?,
            None => bitarr![0],
        }
        .resize(bitsize, bitstate![0]);
        let inputs = args.inputs.unwrap_or(2);
        //Theres gotta be a cleaner way to do this
        let mut port_asgms = [None; 64];
        let num_legs = args.num_legs.unwrap_or(2);
        if let Some(ref assignments) = args.port_assignments {
            let limit = assignments.len().min(64);
            port_asgms[..limit].copy_from_slice(&assignments[..limit]);
        } else {
            port_asgms[0] = Some(0);
            port_asgms[1] = Some(1);
        }

        let inner: PhysicalComponentEnum = match args.component_type.as_str() {
            "PIN" => func::Pin::new(bitsize, args.is_input.unwrap_or(false), orient).into(),
            "CONSTANT" => func::Constant::new(bit_array, orient).into(),
            "SPLITTER" => {
                func::Splitter::new(port_asgms, num_legs, bitsize, orient, handedness).into()
            }
            "POWER" => func::Power.into(),
            "GROUND" => func::Ground.into(),
            "TUNNEL" => func::Tunnel::new(orient).into(),
            "PROBE" => func::Probe::new(orient, bitsize).into(),
            "MUX" => func::Mux::new(bitsize, selsize, orient, handedness).into(),
            "DEMUX" => func::Demux::new(bitsize, selsize, orient, handedness).into(),
            "DECODER" => func::Decoder::new(selsize, orient, handedness).into(),
            "TEXT" => func::Text.into(),
            "SUBCIRCUIT" => todo!(), //func::Subcircuit::new(circuit_key).into(), // TODO: I don't believe this is correct
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

        let label = args.label.as_deref().unwrap_or_default();
        let label_location = args.label_orientation.map(From::from).unwrap_or_default();
        let origin = args.pos.try_into().context("location was out of bounds")?;
        Ok(AddComponentArgs {
            inner,
            label,
            label_location,
            origin,
        })
    }
}

#[napi(object)]
pub struct TransientComponentState {
    pub backend_key: JsKey,
    pub pos: Location,
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
