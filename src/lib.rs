extern crate wasm_bindgen;
extern crate web_sys;
extern crate googleprojection;
extern crate uuid;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate optional_struct;

mod utils;

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use web_sys::console;
use std::f64;

lazy_static! {
    static ref ALL_POINTS: Mutex<Vec<UniqueMarker>> = Mutex::new(Vec::new());
    static ref CLUSTERS: Mutex<Vec<Cluster>> = Mutex::new(Vec::new());
    static ref ZOOM: AtomicUsize = AtomicUsize::new(0);
    static ref CONFIG: Mutex<Config> = Mutex::new(Config {
        grid_size: 60.0,
        average_center: false,
        log_time: false,
    });
}

#[derive(Debug, OptionalStruct)]
#[optional_derive(Deserialize)]
pub struct Config {
    grid_size: f64,
    average_center: bool,
    log_time: bool,
}

#[wasm_bindgen]
impl Config {

}

#[wasm_bindgen]
pub fn configure(config: &JsValue) {
    utils::set_panic_hook();

    let new_config: OptionalConfig = config.into_serde().unwrap();
    CONFIG.lock().unwrap().apply_options(new_config);
}

// Cluster struct
// TODO: Optionally return markers? https://serde.rs/field-attrs.html#skip_serializing_if
#[derive(Debug, Serialize, Clone)]
pub struct Cluster {
    uuid: Uuid,
    size: u32,
    center: Marker,
    markers: Vec<UniqueMarker>,
    bounds: Bounds,
}

#[wasm_bindgen]
impl Cluster {
    fn add_marker(&mut self, new_point: &UniqueMarker, zoom: usize) {
        if self.markers.contains(new_point) {
            return;
        }
        self.size += 1;
        self.markers.push(new_point.clone());
        if CONFIG.lock().unwrap().average_center {
            self.center.lat = ((self.center.lat * f64::from(self.size)) + new_point.lat) / f64::from(self.size + 1);
            self.center.lng = ((self.center.lng * f64::from(self.size)) + new_point.lng) / f64::from(self.size + 1);
            self.calculate_bounds(zoom)
        }
    }

    fn calculate_bounds(&mut self, zoom: usize) {
        self.bounds = calculate_extended_bounds(&Bounds {
            north: self.center.lat,
            east: self.center.lng,
            south: self.center.lat,
            west: self.center.lng
        }, zoom);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bounds {
    north: f64,
    east: f64,
    south: f64,
    west: f64,
}

#[wasm_bindgen]
impl Bounds {
    fn contains(&self, point: &UniqueMarker) -> bool {
        self.north > point.lat &&
        self.east > point.lng &&
        self.south < point.lat &&
        self.west < point.lng
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Marker {
    lat: f64,
    lng: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct UniqueMarker {
    lat: f64,
    lng: f64,
    #[serde(skip)]
    uuid: Uuid,
}

impl From<&Marker> for UniqueMarker {
    fn from(point: &Marker) -> Self {
        UniqueMarker {
            lat: point.lat,
            lng: point.lng,
            uuid: Uuid::new_v4(),
        }
    }
}

impl PartialEq for UniqueMarker {
    fn eq(&self, other: &UniqueMarker) -> bool {
        self.uuid == other.uuid
    }
}

#[wasm_bindgen(js_name = addMarkers)]
pub fn add_markers(markers_val: &JsValue) {
    utils::set_panic_hook();
    // TODO see if .extend() is faster/better than .append() ?
    let markers: &mut Vec<Marker> = &mut markers_val.into_serde().unwrap();
    ALL_POINTS.lock().unwrap().append(&mut markers.iter().map(UniqueMarker::from).collect::<Vec<_>>());
}

#[wasm_bindgen(js_name = clusterMarkersInBounds)]
pub fn cluster_markers_in_bounds(bounds_val: &JsValue, zoom: usize) -> JsValue {
    utils::set_panic_hook();
    let log_time = CONFIG.lock().unwrap().log_time;

    let map_bounds: Bounds = calculate_extended_bounds(&bounds_val.into_serde().unwrap(), zoom);
    if log_time {
        console::time_end_with_label("into-wasm");
        console::time_with_label("clustering");
    }
    let clusters = &mut CLUSTERS.lock().unwrap();
    if ZOOM.swap(zoom, Ordering::Relaxed) != zoom {
        clusters.clear();
    }
    cluster_markers(clusters, &ALL_POINTS.lock().unwrap(), &map_bounds, zoom);
    if log_time {
        console::time_end_with_label("clustering");
        console::time_with_label("out-of-wasm");
    }
    JsValue::from_serde(&clusters.to_vec()).unwrap()
}

pub fn cluster_markers(existing_clusters: &mut Vec<Cluster>, markers: &[UniqueMarker], map_bounds: &Bounds, zoom: usize) {
    for point in markers.iter() {
        if map_bounds.contains(point) {
            add_to_closest_cluster(existing_clusters, point, zoom);
        }
    }
}

pub fn add_to_closest_cluster(clusters: &mut Vec<Cluster>, new_point: &UniqueMarker, zoom: usize) {
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
    // TODO make more idiomatic?
    // cluster_index_to_add_to = clusters.iter().min_by_key(|cluster| distance_between_markers(&cluster.get_center(), new_point));

    if cluster_index_to_add_to.is_some() && clusters[cluster_index_to_add_to.unwrap()].bounds.contains(&new_point) {
        let index = cluster_index_to_add_to.unwrap();
        clusters[index].add_marker(new_point, zoom);
    } else {
        clusters.push(Cluster {
            uuid: Uuid::new_v4(),
            size: 1,
            center: Marker{
                lat: new_point.lat,
                lng: new_point.lng
            },
            markers: vec![new_point.clone()],
            bounds: calculate_extended_bounds(&Bounds {
                north: new_point.lat,
                east: new_point.lng,
                south: new_point.lat,
                west: new_point.lng
            }, zoom)
        })
    };
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

pub fn calculate_extended_bounds(bounds: &Bounds, zoom: usize) -> Bounds {
    let mut north_east_pix = googleprojection::from_ll_to_subpixel(&(bounds.east, bounds.north), zoom).unwrap();
    let mut south_west_pix = googleprojection::from_ll_to_subpixel(&(bounds.west, bounds.south), zoom).unwrap();

    let grid_size = CONFIG.lock().unwrap().grid_size;

    north_east_pix.0 += grid_size;
    north_east_pix.1 -= grid_size;

    south_west_pix.0 -= grid_size;
    south_west_pix.1 += grid_size;
    
    // println!("ne0 {}, ne1 {}, sw0 {}, sw1 {}", north_east_pix.0, north_east_pix.1, south_west_pix.0, south_west_pix.1);
    let north_east_latlng = googleprojection::from_pixel_to_ll(&(north_east_pix.0, north_east_pix.1), zoom).unwrap();
    let south_west_latlng = googleprojection::from_pixel_to_ll(&(south_west_pix.0, south_west_pix.1), zoom).unwrap();

    Bounds {
        north: north_east_latlng.1,
        east: north_east_latlng.0,
        south: south_west_latlng.1,
        west: south_west_latlng.0,
    }
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
        let sample_markers = vec![ Marker { lat: 43.0, lng: -79.0 }; 5 ].iter().map(UniqueMarker::from).collect::<Vec<_>>();

        let clustered = &mut Vec::new();
        cluster_markers(clustered, &sample_markers, &DEFAULT_BOUNDS, DEFAULT_ZOOM);
        let cluster_point_count = clustered.iter().fold(0, |sum, ref x| sum + x.size );
        assert_eq!(sample_markers.len() as u32, cluster_point_count);
    }

    #[test]
    fn add_some_markers_to_a_cluster() {
        let mut sample_clusters: Vec<Cluster> = Vec::new();
        let p1 = UniqueMarker::from(&SAMPLE_POINT);
        let p2 = UniqueMarker::from(&SAMPLE_POINT);

        add_to_closest_cluster(&mut sample_clusters, &p1, DEFAULT_ZOOM);

        assert_eq!(sample_clusters.len(), 1);
        assert_eq!(sample_clusters[0].size, 1);
        assert_eq!(sample_clusters[0].center.lat, SAMPLE_POINT.lat);
        assert_eq!(sample_clusters[0].center.lng, SAMPLE_POINT.lng);

        add_to_closest_cluster(&mut sample_clusters, &p2, DEFAULT_ZOOM);

        assert_eq!(sample_clusters.len(), 1);
        assert_eq!(sample_clusters[0].size, 2);
        assert_eq!(sample_clusters[0].center.lat, SAMPLE_POINT.lat);
        assert_eq!(sample_clusters[0].center.lng, SAMPLE_POINT.lng);
    }

    #[test]
    fn test_10000_markers() {
        let sample_markers = vec![ Marker { lat: 43.0, lng: -79.0 }; 10000 ].iter().map(UniqueMarker::from).collect::<Vec<_>>();
        
        let clustered = &mut Vec::new();
        cluster_markers(clustered, &sample_markers, &DEFAULT_BOUNDS, DEFAULT_ZOOM);
        assert_eq!(clustered.len(), 1);
        assert_eq!(clustered.get(0).unwrap().size, 10000);
    }

    #[test]
    fn bounds_get_extended() {
        let bounds = Bounds {
            north: 43.6532,
            east: -79.3832,
            south: 43.6532,
            west: -79.3832,
        };

        let extended_bounds = calculate_extended_bounds(&bounds, DEFAULT_ZOOM);

        assert!(bounds.north < extended_bounds.north);
        assert!(bounds.east < extended_bounds.east);
        assert!(bounds.south > extended_bounds.south);
        assert!(bounds.west > extended_bounds.west);
    }
}