extern crate webassembly_marker_clusterer;

use std::fs::File;
use std::io::BufReader;

use structs::{ bounds::Bounds, marker::Marker, unique_marker::UniqueMarker };

pub static DEFAULT_BOUNDS: Bounds = Bounds {
    north: 45.0,
    east: -75.0,
    south: 40.0,
    west: -81.0,
};

pub fn get_sample_markers() -> Vec<Marker> {
  let mut rdr = csv::Reader::from_reader(BufReader::new(File::open("benches/points.csv").unwrap()));
  rdr.deserialize::<Marker>()
    .map(|row| row.unwrap())
    .collect::<Vec<_>>()
}

pub fn get_unique_sample_markers() -> Vec<UniqueMarker> {
  get_sample_markers()
    .iter()
    .map(UniqueMarker::from)
    .collect::<Vec<_>>()
}
