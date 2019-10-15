#![cfg(target_arch = "wasm32")]

#[macro_use]
extern crate criterion;
extern crate webassembly_marker_clusterer;
extern crate wasm_bindgen;

use wasm_bindgen::prelude::JsValue;

use criterion::{ Criterion, black_box, BatchSize };

use webassembly_marker_clusterer::*;
use structs::marker::Marker;

mod utils;
use utils::markers::{get_sample_markers, get_unique_sample_markers};

fn serialization(c: &mut Criterion) {
  let unique_sample_markers = get_unique_sample_markers();

  let mut serialize = c.benchmark_group("serialize");
  serialize.bench_function("default-serde", |b| b.iter(|| JsValue::from_serde(black_box(&unique_sample_markers)).unwrap()));
  serialize.bench_function("serde-wasm-bindgen", |b| b.iter(|| serde_wasm_bindgen::to_value(black_box(&unique_sample_markers)).unwrap()));
  serialize.finish();

  let sample_markers = get_sample_markers();
  let mut deserialize = c.benchmark_group("deserialize");
  deserialize.bench_function("serde-wasm-bindgen", |b| b.iter_batched(
    || JsValue::from_serde(&sample_markers).unwrap(),
    |js_value| serde_wasm_bindgen::from_value::<Vec<Marker>>(black_box(js_value)).unwrap(),
    BatchSize::SmallInput
  ));
  deserialize.bench_function("default-serde", |b| b.iter_batched(
    || JsValue::from_serde(&sample_markers).unwrap(),
    |js_value| black_box(js_value).into_serde::<Vec<Marker>>().unwrap(),
    BatchSize::SmallInput
  ));
  deserialize.finish();
}

criterion_group!(serializing, serialization);
criterion_main!(serializing);
