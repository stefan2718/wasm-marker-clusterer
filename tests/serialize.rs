extern crate wasm_bindgen;
extern crate wasm_bindgen_test;
extern crate utilities;
extern crate webassembly_marker_clusterer;

use wasm_bindgen_test::*;
use utilities::{ get_sample_markers };
use wasm_bindgen::prelude::JsValue;
use webassembly_marker_clusterer::structs::marker::Marker;

// This runs a unit test in the browser, so it can use browser APIs.
// wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn serialize_deserialize() {
  let markers = get_sample_markers();
  let js_value = JsValue::from_serde(&markers).unwrap();
  let ser_markers: Vec<Marker> = js_value.into_serde().unwrap();
  assert_eq!(markers.len(), ser_markers.len());
}
