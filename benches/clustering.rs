#[macro_use]
extern crate criterion;
extern crate webassembly_marker_clusterer;

use std::fs::File;
use std::io::BufReader;

use criterion::Criterion;
use criterion::black_box;
use criterion::BenchmarkId;
// use criterion::Throughput;
use criterion::BatchSize;

use webassembly_marker_clusterer::*;
use structs::bounds::Bounds;
use structs::marker::Marker;
use structs::unique_marker::UniqueMarker;

// static DEFAULT_ZOOM: usize = 8;
static DEFAULT_BOUNDS: Bounds = Bounds {
    north: 45.0,
    east: -75.0,
    south: 40.0,
    west: -81.0,
};


fn criterion_benchmark(c: &mut Criterion) {
  let mut rdr = csv::Reader::from_reader(BufReader::new(File::open("benches/points.csv").unwrap()));
  let sample_markers = rdr.deserialize::<Marker>()
             .map(|row| UniqueMarker::from(&row.unwrap()))
             .collect::<Vec<_>>();
  
  let mut group = c.benchmark_group("cluster 10000 real");
  for zoom in 7..12 {
    group.bench_with_input(BenchmarkId::from_parameter(zoom), &zoom, |b, &zoom_cur| b.iter_batched_ref(
      || sample_markers.to_vec(),
      |mut markers| cluster_markers(black_box(&mut Vec::new()), black_box(&mut markers), black_box(&DEFAULT_BOUNDS), zoom_cur),
      BatchSize::SmallInput
    ));
  }
}

criterion_group!{
  name = benches;
  config = Criterion::default().sample_size(10);
  targets = criterion_benchmark
}
criterion_main!(benches);
