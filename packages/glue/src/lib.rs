#![deny(clippy::all)]

use std::sync::Mutex;
use std::sync::LazyLock;
use circuitsim_engine::middle_end::{ComponentKey, MiddleRepr, UIKey, func::{self, PhysicalComponentEnum}};
use circuitsim_engine::engine::func::GateKind;
use circuitsim_engine::engine::FunctionKey;
use napi::JsNumber;
use napi_derive::napi;
use slotmap::KeyData;
use napi::bindgen_prelude::BigInt;
static REPR: LazyLock<Mutex<MiddleRepr>> = LazyLock::new(|| Mutex::new(MiddleRepr::new()));


/// Creates a new circuit and returns its key as an i64 for JS.
#[napi]
pub fn create_circuit()-> Result<BigInt, napi::Error> {
  let mut repr = REPR.lock().unwrap();
  let key = repr.add_circuit();
  Ok(key_to_bigint(key))
}

#[napi]
pub fn add_component(circuit_key: BigInt, properties:CircuitComponent) -> Result<BigInt, napi::Error>{
  let mut rep = REPR.lock().unwrap();
  let key = bigint_to_key(&circuit_key).ok_or_else(|| napi::Error::from_reason("invalid circut key"))?;
  if !rep.has_circuit(key) {return Err(napi::Error::from_reason("Circuit not found"));}
  let mut circuit = rep.circuit(key);

   let c:PhysicalComponentEnum = match properties.component_type.as_str() {
    "AND" => PhysicalComponentEnum::Gate(func::Gate::new(GateKind::And, properties.bitsize,  properties.inputs)),
    "OR" => PhysicalComponentEnum::Gate(func::Gate::new(GateKind::Or, properties.bitsize,  properties.inputs)),
    "NAND" => PhysicalComponentEnum::Gate(func::Gate::new(GateKind::Nand, properties.bitsize,  properties.inputs)),
    "NOR" => PhysicalComponentEnum::Gate(func::Gate::new(GateKind::Nor, properties.bitsize,  properties.inputs)),
    "XNOR" => PhysicalComponentEnum::Gate(func::Gate::new(GateKind::Xor, properties.bitsize,  properties.inputs)),
    "XOR" => PhysicalComponentEnum::Gate(func::Gate::new(GateKind::Xor, properties.bitsize,  properties.inputs)),
    "NOT" => PhysicalComponentEnum::Not(func::Not::new(properties.bitsize)),
    "BUFFER" => PhysicalComponentEnum::TriState(func::TriState::new(properties.bitsize)),
    _ => return Err(napi::Error::from_reason("Unknown component type")),
   };
  let cmpkey = circuit.add_component(c, &properties.label,  (properties.x, properties.y)).map_err(|_| napi::Error::from_reason("Component edit failed"))?;
  let big = match cmpkey {//unwrap the component key into either type and convert to bigint
  ComponentKey::Function(k) => key_to_bigint(k),
  ComponentKey::UI(k) => key_to_bigint(k),
};  Ok(big)
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




/**
 * add/remove wire
 * set input value
 * propogate
 * get circuit state
 * 
 */

 #[napi]
 pub fn get_circuit_state(circuit_key: BigInt) -> Result<CircuitState, napi::Error>{
  //Get the circuit
  let mut rep = REPR.lock().unwrap();
  let key = bigint_to_key(&circuit_key).ok_or_else(|| napi::Error::from_reason("invalid circut key"))?;
  if !rep.has_circuit(key) {return Err(napi::Error::from_reason("Circuit not found"));}

  let  circuit = rep.circuit(key);

  //iterate through all components and get their state
  let mut component_states = Vec::new();
  for (key, state) in circuit.get_component_states() {
    let big_int = key_to_bigint(key);

    //get num ports and iterate through them to get values and states
    let num_ports = state.get_num_ports();
    let port_values = (0..num_ports).map(|i| state.get_port(i).to_string()).collect();
    component_states.push(ComponentState { key: big_int, port_values });
  }



  return Ok(CircuitState { components: component_states })
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


  

#[napi(object)]
pub struct CircuitComponent {
  pub component_type:String, 
  pub label:String, 
  pub bitsize:u8,
  pub inputs:u8,
  pub x:u32,
  pub y:u32

}
#[napi(object)]
pub struct CircuitState {
  pub components: Vec<ComponentState>
  //wires


}

#[napi(object)]
pub struct ComponentState{
  pub key:BigInt,
  pub port_values: Vec<String>,//0,1,Z,X low, high, impedence, unkown

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
