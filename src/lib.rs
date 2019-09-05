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
use std::f64::consts::PI;

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
// TODO: Optionally return markers?
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
    let mut north_east_pix = from_ll_to_pixel(&(bounds.east, bounds.north), zoom).unwrap();
    let mut south_west_pix = from_ll_to_pixel(&(bounds.west, bounds.south), zoom).unwrap();

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

pub fn from_ll_to_pixel(ll: &(f64, f64), zoom: usize) -> Option<(f64, f64)> {
    if 30 > zoom {
        let c = 256.0 * 2.0_f64.powi(zoom as i32);
        let bc = c / 360.0;
        let cc = c / (2.0 * PI);

        let d = c / 2.0;
        let e = d + ll.0 * bc;
        let f = ll.1.to_radians().sin().max(-0.9999).min(0.9999);
        let g = d + 0.5 * ((1.0 + f) / (1.0 - f)).ln() * -cc;

        Some((e, g))
    } else {
        None
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

    // zoom level is (index + 3) 
    static GMAP_BOUNDS: [Bounds; 17] = [
        Bounds {north: 50.800061065188856, east: -68.83632499999999, south: 35.542543366259075, west: -89.93007499999999},
        Bounds {north: 47.34741387849921, east: -74.10976249999999, south: 39.71693995491094, west: -84.65663749999999},
        Bounds {north: 45.530626397270055, east: -76.74648124999999, south: 41.71519339348616, west: -82.01991874999999},
        Bounds {north: 44.599495541698985, east: -78.06484062499999, south: 42.69175511293576, west: -80.70155937499999},
        Bounds {north: 44.12824279122392, east: -78.72402031249999, south: 43.17436960409823, west: -80.04237968749999},
        Bounds {north: 43.891195023324286, east: -79.05361015624999, south: 43.414258058734866, west: -79.71278984374999},
        Bounds {north: 43.77231589906095, east: -79.21840507812499, south: 43.5338473704056, west: -79.54799492187499},
        Bounds {north: 43.712787543711634, east: -79.30080253906249, south: 43.59355327358944, west: -79.46559746093749},
        Bounds {north: 43.68300117005328, east: -79.34200126953124, south: 43.623384034267886, west: -79.42439873046874},
        Bounds {north: 43.66810243453164, east: -79.36260063476561, south: 43.63829386654838, west: -79.40379936523436},
        Bounds {north: 43.66065167963645, east: -79.3729003173828, south: 43.64574739563353, west: -79.39349968261718},
        Bounds {north: 43.65692595541019, east: -79.3780501586914, south: 43.6494738134073, west: -79.38834984130858},
        Bounds {north: 43.65506300660299, east: -79.38062507934569, south: 43.65133693560138, west: -79.38577492065428},
        Bounds {north: 43.65413151052596, east: -79.38191253967284, south: 43.652268475025124, west: -79.38448746032714},
        Bounds {north: 43.653665757069085, east: -79.38255626983641, south: 43.652734239318676, west: -79.38384373016356},
        Bounds {north: 43.653432878986074, east: -79.3828781349182, south: 43.65296712011086, west: -79.38352186508178},
        Bounds {north: 43.6533164396059, east: -79.3830390674591, south: 43.653083560168305, west: -79.38336093254088}
    ];

    #[test]
    fn compare_bound_extension_to_gmap() {
        let bounds = Bounds {
            north: 43.6532,
            east: -79.3832,
            south: 43.6532,
            west: -79.3832,
        };

        println!("zoom, north, east, south, west");
        for (i, g_bounds) in GMAP_BOUNDS.iter().enumerate() {
            let zoom = i + 3;
            let extended_bounds = calculate_extended_bounds(&bounds, zoom);
            bounds_error(&g_bounds, &extended_bounds, zoom);
        }
    }

    fn bounds_error(gmap: &Bounds, wasm: &Bounds, zoom: usize) {
        print!("{:02}, ", zoom);
        print!("{:11.8}, ", percent_error(gmap.north, wasm.north));
        print!("{:11.8}, ", percent_error(gmap.east, wasm.east));
        print!("{:11.8}, ", percent_error(gmap.south, wasm.south));
        print!("{:11.8}  ", percent_error(gmap.west, wasm.west));
        println!();
    }

    fn percent_error(expected: f64, achieved: f64) -> f64 {
        (achieved - expected)/expected.abs() * 100.0
    }
}