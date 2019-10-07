extern crate wasm_bindgen;
extern crate web_sys;
extern crate uuid;
extern crate serde_wasm_bindgen;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate optional_struct;

pub mod structs;
use structs::{ bounds::Bounds, cluster::Cluster, marker::Marker, unique_marker::UniqueMarker };

pub mod config;
use config::{ Config, OptionalConfig };

mod utils;
use utils::calculate_extended_bounds;

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashSet;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use web_sys::console;
use std::f64;

lazy_static! {
    static ref ALL_POINTS: Mutex<Vec<UniqueMarker>> = Mutex::new(Vec::new());
    static ref CLUSTERS: Mutex<Vec<Cluster>> = Mutex::new(Vec::new());
    static ref ZOOM: AtomicUsize = AtomicUsize::new(0);
    static ref CONFIG: Mutex<Config> = Mutex::new(Config::default());
}

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    Ok(())
}

#[wasm_bindgen]
pub fn configure(config: JsValue) {
    let new_config: OptionalConfig = serde_wasm_bindgen::from_value(config).unwrap();
    CONFIG.lock().unwrap().apply_options(new_config);
}

#[wasm_bindgen(js_name = addMarkers)]
pub fn add_markers(markers_val: JsValue) {
    let markers: &mut Vec<Marker> = &mut serde_wasm_bindgen::from_value(markers_val).unwrap();
    // TODO see if .extend() is faster/better than .append() ?
    ALL_POINTS.lock().unwrap().append(&mut markers.iter().map(UniqueMarker::from).collect::<Vec<_>>());
}

#[wasm_bindgen(js_name = clusterMarkersInBounds)]
pub fn cluster_markers_in_bounds(bounds_val: JsValue, zoom: usize) -> JsValue {
    let config = CONFIG.lock().unwrap();

    let map_bounds: Bounds = calculate_extended_bounds(&serde_wasm_bindgen::from_value(bounds_val).unwrap(), zoom, config.grid_size);
    if config.log_time {
        console::time_end_with_label("into-wasm");
        console::time_with_label("clustering");
    }
    let clusters = &mut CLUSTERS.lock().unwrap();
    if ZOOM.swap(zoom, Ordering::Relaxed) != zoom {
        clusters.clear();
        for marker in ALL_POINTS.lock().unwrap().iter_mut() {
            marker.is_added = false;
        }
    }
    let uuids_modified = cluster_markers(clusters, &mut ALL_POINTS.lock().unwrap(), &map_bounds, zoom, &config);
    if config.log_time {
        console::time_end_with_label("clustering");
        console::time_with_label("out-of-wasm");
    }

    let vec = if !config.only_return_modified_clusters {
        clusters.to_vec()
    } else {
        clusters.iter()
                .filter(|c| uuids_modified.contains(&c.uuid))
                .cloned()
                .collect::<Vec<_>>()
    };
    serde_wasm_bindgen::to_value(&vec).unwrap()
}

#[wasm_bindgen]
pub fn clear() {
    ALL_POINTS.lock().unwrap().clear();
    CLUSTERS.lock().unwrap().clear();
}

pub fn cluster_markers(existing_clusters: &mut Vec<Cluster>, markers: &mut Vec<UniqueMarker>, map_bounds: &Bounds, zoom: usize, config: &Config) -> HashSet<Uuid> {
    let mut clusters_modified = HashSet::new();
    for point in markers.iter_mut() {
        if !point.is_added && map_bounds.contains(point) {
            point.is_added = true;
            let closest_cluster = add_to_closest_cluster(existing_clusters, point, zoom, config.average_center, config.grid_size);
            if config.only_return_modified_clusters {
                clusters_modified.insert(closest_cluster);
            }
        }
    }
    clusters_modified
}

pub fn add_to_closest_cluster(clusters: &mut Vec<Cluster>, new_point: &UniqueMarker, zoom: usize, average_center: bool, grid_size: f64) -> Uuid {
    let mut current_distance: f64;
    let mut least_distance = 40000.0; // Some large number
    let mut cluster_index_to_add_to: Option<usize> = None;
    for (i, cluster) in clusters.iter().enumerate() {
        current_distance = distance_between_markers(&cluster.center, new_point);
        if current_distance < least_distance {
            least_distance = current_distance;
            cluster_index_to_add_to = Some(i);
        }
    }

    if cluster_index_to_add_to.is_some() && clusters[cluster_index_to_add_to.unwrap()].bounds.contains(&new_point) {
        let index = cluster_index_to_add_to.unwrap();
        clusters[index].add_marker(new_point, zoom, average_center, grid_size);
        clusters[index].uuid
    } else {
        let uuid = Uuid::new_v4();
        clusters.push(Cluster {
            uuid,
            size: 1,
            center: Marker{
                lat: new_point.lat,
                lng: new_point.lng
            },
            markers: vec![new_point.clone()],
            bounds: Bounds::from_point(new_point.lat, new_point.lng, zoom, grid_size)
        });
        uuid
    }
}

pub fn distance_between_markers(p1: &Marker, p2: &UniqueMarker) -> f64 {
    let earth_radius_kilometer = 6371.0_f64;

    let delta_latitude = (p1.lat - p2.lat).to_radians();
    let delta_longitude = (p1.lng - p2.lng).to_radians();

    let central_angle_inner = (delta_latitude / 2.0).sin().powi(2)
        + p1.lat.to_radians().cos() * p2.lat.to_radians().cos() * (delta_longitude / 2.0).sin().powi(2);
    let central_angle = 2.0 * central_angle_inner.sqrt().asin();

    earth_radius_kilometer * central_angle
}

#[cfg(test)]
mod tests {
    use super::*;

    static SAMPLE_POINT: Marker = Marker { lat: 43.0, lng: -79.0 };
    static DEFAULT_ZOOM: usize = 8;
    static DEFAULT_BOUNDS: Bounds = Bounds {
        north: 45.0,
        east: -75.0,
        south: 40.0,
        west: -81.0,
    };

    #[test]
    fn clusters_include_all_markers() {
        let mut sample_markers = vec![ Marker { lat: 43.0, lng: -79.0 }; 5 ].iter().map(UniqueMarker::from).collect::<Vec<_>>();

        let clustered = &mut Vec::new();
        cluster_markers(clustered, &mut sample_markers, &DEFAULT_BOUNDS, DEFAULT_ZOOM, &Config::default());
        let cluster_point_count = clustered.iter().fold(0, |sum, ref x| sum + x.size );
        assert_eq!(sample_markers.len() as u32, cluster_point_count);
    }

    #[test]
    fn add_some_markers_to_a_cluster() {
        let mut sample_clusters: Vec<Cluster> = Vec::new();
        let p1 = UniqueMarker::from(&SAMPLE_POINT);
        let p2 = UniqueMarker::from(&SAMPLE_POINT);

        add_to_closest_cluster(&mut sample_clusters, &p1, DEFAULT_ZOOM, false, 60.0);

        assert_eq!(sample_clusters.len(), 1);
        assert_eq!(sample_clusters[0].size, 1);
        assert!((sample_clusters[0].center.lat - SAMPLE_POINT.lat).abs() < f64::EPSILON);
        assert!((sample_clusters[0].center.lng - SAMPLE_POINT.lng).abs() < f64::EPSILON);

        add_to_closest_cluster(&mut sample_clusters, &p2, DEFAULT_ZOOM, false, 60.0);

        assert_eq!(sample_clusters.len(), 1);
        assert_eq!(sample_clusters[0].size, 2);
        assert!((sample_clusters[0].center.lat - SAMPLE_POINT.lat).abs() < f64::EPSILON);
        assert!((sample_clusters[0].center.lng - SAMPLE_POINT.lng).abs() < f64::EPSILON);
    }

    #[test]
    fn test_1000_markers() {
        let mut sample_markers = vec![ Marker { lat: 43.0, lng: -79.0 }; 1000 ].iter().map(UniqueMarker::from).collect::<Vec<_>>();
        
        let clustered = &mut Vec::new();
        cluster_markers(clustered, &mut sample_markers, &DEFAULT_BOUNDS, DEFAULT_ZOOM, &Config::default());
        assert_eq!(clustered.len(), 1);
        assert_eq!(clustered.get(0).unwrap().size, 1000);
    }

    #[test]
    fn bounds_get_extended() {
        let bounds = Bounds {
            north: 43.6532,
            east: -79.3832,
            south: 43.6532,
            west: -79.3832,
        };

        let extended_bounds = calculate_extended_bounds(&bounds, DEFAULT_ZOOM, 60.0);

        assert!(bounds.north < extended_bounds.north);
        assert!(bounds.east < extended_bounds.east);
        assert!(bounds.south > extended_bounds.south);
        assert!(bounds.west > extended_bounds.west);
    }
}