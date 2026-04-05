#![deny(clippy::all)]

use std::sync::{LazyLock, Mutex};

use circuitsim_engine::{engine::{ FunctionKey, func::{bitsize_from_u8, gateinputs_from_u8}}, middle_end::{ComponentKey, MiddleRepr, UIKey, func::{ Gate, Orientation}}};
use circuitsim_engine::engine::func::{GateKind};
use napi_derive::napi;
use slotmap::KeyData;
use napi::bindgen_prelude::BigInt;
static REPR: LazyLock<Mutex<MiddleRepr>> = LazyLock::new(|| Mutex::new(MiddleRepr::new()));

/// Creates a new circuit and returns its key as an i64 for JS.
#[napi]
pub fn create_circuit(name:String)-> Result<BigInt, napi::Error> {
  let mut repr = REPR.lock().unwrap();
  let key = repr.add_circuit(&name);
  Ok(key_to_bigint(key))
}

#[napi]
pub fn add_component(circuit_key: BigInt, gate_kind:String, bitsize:u8, inputs:u8, orientation:u8,label:String, x:u32, y:u32, label_orientation:u8) -> Result<BigInt, napi::Error>{
   let mut rep = REPR.lock().unwrap();

   let gate_type:GateKind = match gate_kind.as_str() {
    "AND" => GateKind::And,
    "OR" => GateKind::Or,
    "NAND" => GateKind::Nand,
    "NOR" => GateKind::Nor,
    "XNOR" => GateKind::Xnor,
    "XOR" => GateKind::Xor,
    _ => return Err(napi::Error::from_reason("Unknown gate type")),
   };
   let orient:Orientation = match orientation{
    0 => Orientation::North,
    1 => Orientation::South,
    2 => Orientation::East,
    3 => Orientation::West,
    _ => return Err(napi::Error::from_reason("Invalid orientation value")),
   };
   let label_orient: Orientation = match label_orientation {
    0 => Orientation::North,
    1 => Orientation::South,
    2 => Orientation::East,
    3 => Orientation::West,
    _ => return Err(napi::Error::from_reason("Invalid label orientation value")),
   };
   let gate:Gate = Gate::new(gate_type, bitsize_from_u8(bitsize).ok_or_else(|| napi::Error::from_reason("Invalid bit size"))?, gateinputs_from_u8(inputs).ok_or_else(|| napi::Error::from_reason("Invalid gate inputs"))?, orient);
   

   let mut circuit = rep.circuit(bigint_to_key(&circuit_key).ok_or_else(|| napi::Error::from_reason("Invalid circuit key"))?);

  let cmpkey = circuit.add_component(gate, &label, label_orient, (x, y)).map_err(|_| napi::Error::from_reason("Component edit failed"))?;
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