extern crate wasm_bindgen;
extern crate wasm_bindgen_test;
extern crate utilities;
extern crate webassembly_marker_clusterer;
extern crate web_sys;
extern crate easybench_wasm;

use wasm_bindgen_test::*;
use wasm_bindgen::prelude::JsValue;
use utilities::{ get_sample_markers, get_unique_sample_markers, DEFAULT_BOUNDS };
use webassembly_marker_clusterer::{
  add_markers,
  cluster_markers_in_bounds,
  clear_clusters,
  structs::marker::Marker
};
use web_sys::console;

// This runs a unit test in the browser, so it can use browser APIs.
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn serialize_deserialize() {
  let unique_sample_markers = get_unique_sample_markers();
  let run_time = 10.0;

  let serialize_default_serde = easybench_wasm::bench_limit(run_time,
    || JsValue::from_serde(&unique_sample_markers).unwrap());
  let serialize_serde_wasm_bindgen = easybench_wasm::bench_limit(run_time,
    || serde_wasm_bindgen::to_value(&unique_sample_markers).unwrap());

  console::log_1(&format!("serialize/default-serde:        {}", serialize_default_serde).into());
  console::log_1(&format!("serialize/serde-wasm-bindgen:   {}", serialize_serde_wasm_bindgen).into());

  let sample_markers = get_sample_markers();

  let deserialize_default_serde = easybench_wasm::bench_env_limit(run_time,
    JsValue::from_serde(&sample_markers).unwrap(),
    |js_value| js_value.into_serde::<Vec<Marker>>().unwrap());

  let deserialize_serde_wasm_bindgen = easybench_wasm::bench_env_limit(run_time,
    JsValue::from_serde(&sample_markers).unwrap(),
    |js_value| serde_wasm_bindgen::from_value::<Vec<Marker>>(js_value).unwrap());

  console::log_1(&format!("deserialize/default-serde:      {}", deserialize_default_serde).into());
  console::log_1(&format!("deserialize/serde-wasm-bindgen: {}", deserialize_serde_wasm_bindgen).into());
}

#[wasm_bindgen_test]
fn cluster() {
  let sample_markers = get_sample_markers();
  let js = JsValue::from_serde(&sample_markers).unwrap();
  let run_time = 6.0;

  add_markers(js);

  console::log_1(&"zm,    nanos,    R², itrs, smpl".into());
  for zoom in 7..12 {
    let res = easybench_wasm::bench_limit(run_time,
      || {
        let bounds = JsValue::from_serde(&DEFAULT_BOUNDS).unwrap();
        cluster_markers_in_bounds(bounds, zoom);
        clear_clusters();
      });

    console::log_1(&format!("{:2}, {:8.0}, {:.3}, {:4}, {:4}", zoom, res.ns_per_iter, res.goodness_of_fit, res.iterations, res.samples).into());
  }
}