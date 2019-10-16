extern crate webassembly_marker_clusterer;

pub mod data;
use data::MARKERS;

use webassembly_marker_clusterer::structs::{ bounds::Bounds, marker::Marker, unique_marker::UniqueMarker };

pub static DEFAULT_BOUNDS: Bounds = Bounds {
    north: 45.0,
    east: -75.0,
    south: 40.0,
    west: -81.0,
};

pub fn get_sample_markers() -> Vec<Marker> {
  MARKERS.to_vec()
}

pub fn get_unique_sample_markers() -> Vec<UniqueMarker> {
  get_sample_markers()
    .iter()
    .map(UniqueMarker::from)
    .collect::<Vec<_>>()
}
