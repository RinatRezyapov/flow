use std::collections::HashMap;

use crate::dataproperty::*;

pub fn i64_to_bytes(val:i64) -> Vec<u8> { 
  val.to_le_bytes().to_vec()
}

pub fn bytes_to_i64(bytes:&Vec<u8>, off:usize) -> i64{
  i64::from_le_bytes([bytes[off+0], bytes[off+1], bytes[off+2], bytes[off+3], bytes[off+4], bytes[off+5], bytes[off+6], bytes[off+7]])
}

pub fn f64_to_bytes(val:f64) -> Vec<u8> {
  val.to_le_bytes().to_vec()
}

pub fn bytes_to_f64(bytes:&Vec<u8>, off:usize) -> f64{
  f64::from_le_bytes([bytes[off+0], bytes[off+1], bytes[off+2], bytes[off+3], bytes[off+4], bytes[off+5], bytes[off+6], bytes[off+7]])
}

pub fn propertymap_to_bytes(props: HashMap<usize, DataProperty>) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::new();
  for (_key, val) in props {
    bytes.append(&mut val.to_bytes());
  }
  bytes
}

pub fn bytes_to_propertymap(bytes:Vec<u8>, off:usize, len:usize) -> HashMap<usize, DataProperty>{
  let mut map: HashMap<usize, DataProperty> = HashMap::new();
  let n = len - off;
  let mut i = off;
  while i<n {
    let dp:DataProperty = DataProperty::from_bytes(&bytes, i);
    map.insert(dp.id, dp);
    i = i + PROPERTY_SIZE as usize;
  }
  map
}

pub fn propertyvec_to_bytes(props: Vec<DataProperty>) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::new();
  for val in props {
    bytes.append(&mut val.to_bytes());
  }
  bytes
}

pub fn bytes_to_propertyvec(bytes:Vec<u8>, off:usize, len:usize) -> Vec<DataProperty>{
  let mut vec: Vec<DataProperty> = Vec::new();
  let n = len - off;
  let mut i = off;
  while i<n {
    let dp:DataProperty = DataProperty::from_bytes(&bytes, i);
    vec.push(dp);
    i = i + PROPERTY_SIZE as usize;
  }
  vec
}

#[test]
fn verify_test() {
  let f1:f64 = 7.2;
  let bytes:Vec<u8> = f64_to_bytes(f1);
  let f2 = bytes_to_f64(&bytes, 0);
  assert_eq!(f1, f2);
}
