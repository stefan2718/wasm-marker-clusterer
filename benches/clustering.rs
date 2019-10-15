#[macro_use]
extern crate criterion;
extern crate webassembly_marker_clusterer;

use criterion::{ Criterion, black_box, Throughput, BenchmarkId, BatchSize };

use webassembly_marker_clusterer::*;
use config::Config;

mod utils;
use utils::markers::{DEFAULT_BOUNDS, get_unique_sample_markers};

fn clustering_benchmark(c: &mut Criterion) {
  let mut clustering = c.benchmark_group("cluster-10000");
  let sample_markers = get_unique_sample_markers();
  let config = Config::default();

  for (zoom, clusters) in &[(7,1), (8,6), (9,16), (10,51), (11,162)] {
    clustering.throughput(Throughput::Elements((sample_markers.len() / clusters) as u64));
    clustering.bench_with_input(BenchmarkId::from_parameter(zoom), &zoom, |b, &zoom_cur| b.iter_batched_ref(
      || sample_markers.to_vec(),
      |mut markers| cluster_markers(black_box(&mut Vec::new()), black_box(&mut markers), &DEFAULT_BOUNDS, *zoom_cur, black_box(&config)),
      BatchSize::SmallInput
    ));
  }
  clustering.finish();
}

criterion_group!{
  name = clustering;
  config = Criterion::default().sample_size(10);
  targets = clustering_benchmark 
}
criterion_main!(clustering);
