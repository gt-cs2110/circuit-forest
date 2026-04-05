#![deny(clippy::all)]

use std::sync::Mutex;
use once_cell::sync::Lazy;
use circuitsim_engine::middle_end::{ComponentKey, MiddleRepr, func::{self, PhysicalComponentEnum}};
use circuitsim_engine::engine::func::GateKind;
use napi_derive::napi;
use slotmap::KeyData;
use napi::bindgen_prelude::BigInt;
static REPR: Lazy<Mutex<MiddleRepr>> = Lazy::new(|| Mutex::new(MiddleRepr::new()));


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
  let mut circuit = rep.circuit(bigint_to_key(circuit_key).ok_or_else(|| napi::Error::from_reason("Invalid circuit key"))?);
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

/// Returns a debug string describing the circuit for testing.
#[napi]
pub fn debug_circuit(circuit_key: BigInt) -> Result<String, napi::Error> {
  let repr = REPR.lock().unwrap();
  let key = bigint_to_key(circuit_key).ok_or_else(|| napi::Error::from_reason("Invalid circuit key"))?;
  Ok(repr.debug_circuit(key))
}

#[napi]
pub fn print_circuit(circuit_key: BigInt) -> Result<(String), napi::Error> {
  let mut repr = REPR.lock().unwrap();
  let key = bigint_to_key(circuit_key).ok_or_else(|| napi::Error::from_reason("Invalid circuit key"))?;
  Ok(repr.debug_circuit(key))
  
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

pub fn key_to_bigint<K: slotmap::Key>(k: K) -> BigInt {
  let raw = k.data().as_ffi(); // u64
  BigInt::from(raw)
}

fn bigint_to_key<K: slotmap::Key>(b: BigInt) -> Option<K> {
  let (sign, raw, lossless) = b.get_u64();
  if sign || !lossless {
    return None;
  }
  Some(K::from(KeyData::from_ffi(raw)))
}
