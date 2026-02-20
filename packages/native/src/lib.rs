#![deny(clippy::all)]

use std::str::FromStr;

use circuitsim_engine::bitarray::BitArray;
use napi_derive::napi;

#[napi]
pub fn bitwise_and(a: String, b: String) -> Result<String, napi::Error> {
  let a =
    BitArray::from_str(&a).map_err(|_| napi::Error::from_reason("invalid bitstring for 'a'"))?;
  let b =
    BitArray::from_str(&b).map_err(|_| napi::Error::from_reason("invalid bitstring for 'b'"))?;

  Ok((a & b).to_string())
}
