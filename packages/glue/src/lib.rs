use std::num::NonZero;
use std::str::FromStr;
use std::sync::{LazyLock, Mutex};

use circuitsim_engine::bitarray::BitArray;
use circuitsim_engine::engine::FunctionKey;
use circuitsim_engine::engine::func::GateKind;
use circuitsim_engine::engine::state::ValueIssue;
use circuitsim_engine::middle_end::wire::Wire;
use circuitsim_engine::middle_end::{ComponentKey, MiddleRepr, UIKey};
use circuitsim_engine::middle_end::func::{Handedness, Orientation, PhysicalComponentEnum, self};
use napi::bindgen_prelude::BigInt;
use napi_derive::napi;
use slotmap::KeyData;

static REPR: LazyLock<Mutex<MiddleRepr>> = LazyLock::new(|| Mutex::new(MiddleRepr::new()));


/// Creates a new circuit and returns its key as an i64 for JS.
#[napi]
pub fn create_circuit(name:String)-> Result<BigInt, napi::Error> {
  let mut repr = REPR.lock().unwrap();
  let key = repr.add_circuit(&name);
  Ok(key_to_bigint(key))
}

#[napi]
pub fn add_component(args: CreateComponentArgs) -> Result<BigInt, napi::Error>{
   let mut rep = REPR.lock().unwrap();
   let bitsize = args.bitsize.unwrap_or(1);
   let selsize = args.selsize.unwrap_or(1);
let orient:Orientation = match args.orientation.unwrap_or(2) {
    0 => Orientation::North,
    1 => Orientation::South,
    2 => Orientation::East,
    3 => Orientation::West,
    _ => return Err(napi::Error::from_reason("Invalid orientation value")),
   };
   let label_orient: Orientation = match args.labelOrientation.unwrap_or(2) {
    0 => Orientation::North,
    1 => Orientation::South,
    2 => Orientation::East,
    3 => Orientation::West,
    _ => return Err(napi::Error::from_reason("Invalid label orientation value")),
   };
   let handedness: Handedness = match args.handedness.unwrap_or(0) {
    0 => Handedness::TopLeft,
    1 => Handedness::DownRight,
    _ => return Err(napi::Error::from_reason("Invalid handedness value")),
   };
   let bitArray = BitArray::from_str(args.constantValue.unwrap_or("0".to_string()).as_str()).map_err(|_| napi::Error::from_reason("Invalid Constant Value"))?;
   let inputs = args.inputs.unwrap_or(2);
  let component:PhysicalComponentEnum = match args.componentType.as_str() {
    "PIN" => func::Pin::new(bitsize, args.isInput.unwrap_or(false), orient).into(),
    "CONSTANT" => func::Constant::new(bitArray, orient).into(),
    "SPLITTER" => func::Splitter::new(bitsize, orient, handedness).into(),
    "POWER" => func::Power.into(),
    "GROUND" => func::Ground.into(),
    "TUNNEL" => func::Tunnel::new(orient).into(),
    "PROBE" => func::Probe::new( orient).into(),
    "MUX" => func::Mux::new(bitsize, selsize, orient, handedness).into(),
    "DEMUX" => func::Demux::new(bitsize, selsize, orient, handedness).into(),
    "DECODER" => func::Decoder::new(selsize,orient, handedness).into(),
    "TEXT" => func::Text.into(),
    "SUBCIRCUIT" => func::Subcircuit::new(bigint_to_key(&args.circuitKey).ok_or_else(|| napi::Error::from_reason("Invalid circuit key"))?).into(),
    "NOT" => func::Not::new(bitsize,orient).into(),
    "BUFFER" => func::TriState::new(bitsize,orient, handedness).into(),
    "AND" => func::Gate::new(GateKind::And, bitsize, inputs, orient).into(),
    "OR" => func::Gate::new(GateKind::Or, bitsize, inputs, orient).into(),
    "NAND" => func::Gate::new(GateKind::Nand, bitsize, inputs, orient).into(),
    "NOR" => func::Gate::new(GateKind::Nor, bitsize, inputs, orient).into(),
    "XOR" => func::Gate::new(GateKind::Xor, bitsize, inputs, orient).into(),
    "XNOR" => func::Gate::new(GateKind::Xnor, bitsize, inputs, orient).into(),
    _ => return Err(napi::Error::from_reason("Unknown gate type")),
   };
  
   

   

   let mut circuit = rep.circuit(bigint_to_key(&args.circuitKey).ok_or_else(|| napi::Error::from_reason("Invalid circuit key"))?);

  let cmpkey = circuit.add_component(component, &args.label.unwrap_or_default(), label_orient, (args.x, args.y)).map_err(|_| napi::Error::from_reason("Component edit failed"))?;
  let big = match cmpkey {//unwrap the component key into either type and convert to bigint
  ComponentKey::Function(k) => key_to_bigint(k),
  ComponentKey::UI(k) => key_to_bigint(k),
};  
Ok(big)
}
#[napi]
pub fn remove_component(circuit_key: BigInt, component_key: BigInt) -> Result<(), napi::Error> {
    let mut rep = REPR.lock().unwrap();
  let key = bigint_to_key(&circuit_key).ok_or_else(|| napi::Error::from_reason("invalid circut key"))?;
  if !rep.has_circuit(key) {return Err(napi::Error::from_reason("Circuit not found"));}
  let mut circuit = rep.circuit(key);

  

  if let Some(fk) = bigint_to_key::<FunctionKey>(&component_key) {
    if !circuit.has_component(ComponentKey::Function(fk)) {return Err(napi::Error::from_reason("Component not found"));}
    if circuit.remove_component(ComponentKey::Function(fk)).is_ok() {
      return Ok(());
    }
  }

  if let Some(uk) = bigint_to_key::<UIKey>(&component_key) {
    if !circuit.has_component(ComponentKey::UI(uk)) {return Err(napi::Error::from_reason("Component not found"));}
    return circuit.remove_component(ComponentKey::UI(uk))
      .map_err(|_| napi::Error::from_reason("Component removal failed"));
  }

  Err(napi::Error::from_reason("Invalid component key"))
}
#[napi]

pub fn add_wire(cricuit_key: BigInt, wire:TransientWireState)->String{

  let mut rep = REPR.lock().unwrap();
  let key = bigint_to_key(&cricuit_key).unwrap();
  // if !rep.has_circuit(key){return Err(napi::Error::from_reason("Circuit not found"))};
    if !rep.has_circuit(key){return "Circuit Nto Found".to_string()};

  let mut circuit = rep.circuit(key);
  //coordinates are loermost  x and y so smallest value
  let x = std::cmp::min(wire.endpoints[0].x, wire.endpoints[1].x);
  let y= std::cmp::min(wire.endpoints[0].y, wire.endpoints[1].y); 
  // circuit.add_wire(Wire{x, y, length:NonZero::new(wire.length).unwrap(), horizontal:wire.isHorizantal}).map_err(|op| napi::Error::from_reason(op.to_string()))
  let res =  circuit.add_wire(Wire{x, y, length:NonZero::new(wire.length).unwrap(), horizontal:wire.isHorizontal});
  if res.is_err(){
    return res.map_err(|op| op.to_string()).err().unwrap();
  }
  return "".to_string();

}


/// Function Get Transient State, gets the relevant data and state of all components in a circuit

#[napi]
 pub fn getTransientState(circuit_key: BigInt)-> Result<(Vec<TransientComponentState>, Vec<TransientWireState>),napi::Error>{
    let mut component_states: Vec<TransientComponentState> = Vec::new();
    let mut rep = REPR.lock().unwrap();
    let key = bigint_to_key(&circuit_key).ok_or_else(|| napi::Error::from_reason("invalid circut key"))?;
    if !rep.has_circuit(key) {return Err(napi::Error::from_reason("Circuit not found"));}
    let  circuit = rep.circuit(key);

    for (key, state) in circuit.get_component_states() {
      let big_int = key_to_bigint(key);
      let component = circuit.get_component(ComponentKey::Function(key)).map_err(|_| napi::Error::from_reason("Component not found"))?;
      //get num ports and iterate through them to get values and states
      let num_ports = state.get_num_ports();
      let ports: Vec<PortTransientState> = (0..num_ports).map(|i| {
        let (x, y) = component.ports[i];
        let value = state.get_port(i).to_string();
        let valueKey: circuitsim_engine::engine::ValueKey = circuit.get_wire_set().find_key((ComponentKey::Function(key), i)).unwrap();
        let issues  = circuit.get_circuit_state().get_issues(valueKey).iter().map(|issue| {
            match issue{
                ValueIssue::ShortCircuit =>"ShortCircuit".to_string(),
                ValueIssue::OscillationDetected => "OscillationDetected".to_string(),
                ValueIssue::MismatchedBitsizes => "MismatchedBitsize".to_string()
            }
        }).collect();
        PortTransientState { x, y,  value, issues }
      }).collect();
      //check to see if component is a probe or constant to get value for component value field
      let component_value = match component.inner {
        PhysicalComponentEnum::Probe(probe) => Some("0".to_string()), // Need to figure out how to get the actual value of the probe
        PhysicalComponentEnum::Constant(constant) => Some(constant.get_value().to_string()),
        _ => None,
      };
      component_states.push(TransientComponentState { backendKey: big_int.get_i128().0.to_string(), ports, bounds: vec![ Location{x:component.bounds[0].0, y: component.bounds[0].1 }, Location{x:component.bounds[1].0, y: component.bounds[1].1 } ], componentValue: component_value });
    }



    //Get Wire Transient States
    //Middle End has a wire set and tunnel interner'

    //Wire Range Map holds all our horizantal and vertical segments

    let mut wire_states: Vec<TransientWireState> = Vec::new();

    for(wire, value, issues) in circuit.get_wire_states(){
      wire_states.push(TransientWireState{endpoints:vec![Location{x:wire.endpoints()[0].0, y:wire.endpoints()[0].1}, Location{x:wire.endpoints()[1].0, y:wire.endpoints()[1].1}], isHorizontal:wire.horizontal(), length:wire.length(), value:value.to_string(), issues:issues});
    }

   

    Ok((component_states, wire_states))

 }


 

 #[napi]
 pub fn propagate(circuit_key: BigInt) -> Result<(), napi::Error>{
  let mut rep = REPR.lock().unwrap();
  let key = bigint_to_key(&circuit_key).ok_or_else(|| napi::Error::from_reason("invalid circut key"))?;
  if !rep.has_circuit(key) {return Err(napi::Error::from_reason("Circuit not found"));}

  let mut circuit = rep.circuit(key);
  circuit.propagate();
  Ok(())
 }
#[napi]
pub fn print_circuit(circuit_key: BigInt) -> Result<String, napi::Error> {
   let mut rep = REPR.lock().unwrap();
  let key = bigint_to_key(&circuit_key).ok_or_else(|| napi::Error::from_reason("invalid circut key"))?;
  if !rep.has_circuit(key) {return Err(napi::Error::from_reason("Circuit not found"));}
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
pub struct CreateComponentArgs{
  pub circuitKey:BigInt,
  pub componentType:String,
  pub bitsize:Option<u8>,
  pub inputs:Option<u8>,
  pub orientation:Option<u8>,
  pub label:Option<String>,
  pub x:u32,
  pub y:u32,
  pub labelOrientation:Option<u8>,
  pub constantValue:Option<String>,
  pub isInput:Option<bool>,
  pub selsize:Option<u8>,
  pub textContent:Option<String>,
    pub handedness:Option<u8>,

}
#[napi(object)]
pub struct TransientComponentState{
  pub backendKey: String,
  pub ports: Vec<PortTransientState>,
  pub bounds: Vec<Location>,
  pub componentValue: Option<String>//only for probes and constants
}
#[napi(object)]
pub struct TransientWireState{
  pub endpoints: Vec<Location>, 
  pub isHorizontal: bool, 
  pub length: u32,
  pub value: String, 
  pub issues: Vec<String>
}
#[napi(object)]
pub struct PortTransientState{
  pub x: u32,
  pub y: u32,
  pub value:String,
  pub issues:Vec<String>
}
#[napi(object)]
pub struct Location{
  pub x: u32,
  pub y: u32,
}
