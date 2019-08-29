extern crate wasm_bindgen;
extern crate web_sys;
extern crate googleprojection;
extern crate uuid;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

mod utils;

use std::sync::Mutex;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use web_sys::console;
use std::f64;

static GRID_SIZE: f64 = 60.0; // pixels?

lazy_static! {
    static ref ALL_POINTS: Mutex<Vec<Point>> = Mutex::new(Vec::new());
}

// Cluster struct
// - Should maintain list of points in the cluster, but not return that list to JS
#[derive(Debug, Serialize)]
pub struct Cluster {
    uuid: Uuid,
    count: u32,
    center_lat: f64,
    center_lng: f64,
    // points: Vec<&Point>,
    bounds: Bounds,
}

#[wasm_bindgen]
impl Cluster {
    fn get_center(&self) -> Point {
        Point {
            lat: self.center_lat,
            lng: self.center_lng,
            price: 0
        }
    }

    fn add_marker(&mut self, new_point: &Point) {
        self.center_lat = ((self.center_lat * f64::from(self.count)) + new_point.lat) / f64::from(self.count + 1);
        self.center_lng = ((self.center_lng * f64::from(self.count)) + new_point.lng) / f64::from(self.count + 1);
        self.count += 1;
    }

    fn calculate_bounds(&mut self, zoom: usize) {
        self.bounds = calculate_extended_bounds(&Bounds {
            north_east_lat: self.center_lat,
            north_east_lng: self.center_lng,
            south_west_lat: self.center_lat,
            south_west_lng: self.center_lng
        }, zoom);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bounds {
    north_east_lat: f64,
    north_east_lng: f64,
    south_west_lat: f64,
    south_west_lng: f64,
}

#[wasm_bindgen]
impl Bounds {
    fn contains(&self, point: &Point) -> bool {
        self.north_east_lat > point.lat &&
        self.north_east_lng > point.lng &&
        self.south_west_lat < point.lat &&
        self.south_west_lng < point.lng
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Point {
    lat: f64,
    lng: f64,
    price: u32,
}

#[wasm_bindgen]
impl Point {

}

#[wasm_bindgen]
pub fn add_points(points_val: &JsValue) {
    utils::set_panic_hook();
    // TODO see if .extend() is faster/better than .append() ?
    ALL_POINTS.lock().unwrap().append(&mut points_val.into_serde().unwrap());
}

#[wasm_bindgen]
pub fn cluster_points_in_bounds(bounds_val: &JsValue, zoom: usize) -> JsValue {
    utils::set_panic_hook();

    let bounds: Bounds = calculate_extended_bounds(&bounds_val.into_serde().unwrap(), zoom);
    console::time_end_with_label("into-wasm");

    console::time_with_label("clustering");
    let clusters = cluster_points(&ALL_POINTS.lock().unwrap(), &bounds, zoom);
    console::time_end_with_label("clustering");
    console::time_with_label("out-of-wasm");
    return JsValue::from_serde(&clusters).unwrap();
}

pub fn cluster_points(points: &Vec<Point>, bounds: &Bounds, zoom: usize) -> Vec<Cluster> {
    let mut clusters = Vec::new();
    for point in points.iter() {
        if bounds.contains(point) {
            add_to_closest_cluster(&mut clusters, point, zoom);
        }
    }
    clusters
}

pub fn add_to_closest_cluster(clusters: &mut Vec<Cluster>, new_point: &Point, zoom: usize) {
    let mut current_distance: f64;
    let mut least_distance = 40000.0; // Some large number
    let mut cluster_index_to_add_to: Option<usize> = None;
    for (i, cluster) in clusters.iter().enumerate() {
        current_distance = distance_between_points(&cluster.get_center(), new_point);
        if current_distance < least_distance {
            least_distance = current_distance;
            cluster_index_to_add_to = Some(i);
        }
    }
    // TODO make more idiomatic?
    // cluster_index_to_add_to = clusters.iter().min_by_key(|cluster| distance_between_points(&cluster.get_center(), new_point));

    if cluster_index_to_add_to.is_some() && clusters[cluster_index_to_add_to.unwrap()].bounds.contains(&new_point) {
        let index = cluster_index_to_add_to.unwrap();
        clusters[index].add_marker(new_point);
        clusters[index].calculate_bounds(zoom);
    } else {
        clusters.push(Cluster {
            uuid: Uuid::new_v4(),
            count: 1,
            center_lat: new_point.lat,
            center_lng: new_point.lng,
            bounds: calculate_extended_bounds(&Bounds {
                north_east_lat: new_point.lat,
                north_east_lng: new_point.lng,
                south_west_lat: new_point.lat,
                south_west_lng: new_point.lng
            }, zoom)
        })
    };
}

pub fn distance_between_points(p1: &Point, p2: &Point) -> f64 {
    let earth_radius_kilometer = 6371.0_f64;

    let delta_latitude = (p1.lat - p2.lat).to_radians();
    let delta_longitude = (p1.lng - p2.lng).to_radians();

    let central_angle_inner = (delta_latitude / 2.0).sin().powi(2)
        + p1.lat.to_radians().cos() * p2.lat.to_radians().cos() * (delta_longitude / 2.0).sin().powi(2);
    let central_angle = 2.0 * central_angle_inner.sqrt().asin();

    return earth_radius_kilometer * central_angle;
}

pub fn calculate_extended_bounds(bounds: &Bounds, zoom: usize) -> Bounds {
    let mut north_east_pix = googleprojection::from_ll_to_pixel(&(bounds.north_east_lng, bounds.north_east_lat), zoom).unwrap();
    let mut south_west_pix = googleprojection::from_ll_to_pixel(&(bounds.south_west_lng, bounds.south_west_lat), zoom).unwrap();

    north_east_pix.0 += GRID_SIZE;
    north_east_pix.1 -= GRID_SIZE;

    south_west_pix.0 -= GRID_SIZE;
    south_west_pix.1 += GRID_SIZE;
    
    // println!("ne0 {}, ne1 {}, sw0 {}, sw1 {}", north_east_pix.0, north_east_pix.1, south_west_pix.0, south_west_pix.1);
    let north_east_latlng = googleprojection::from_pixel_to_ll(&(north_east_pix.0, north_east_pix.1), zoom).unwrap();
    let south_west_latlng = googleprojection::from_pixel_to_ll(&(south_west_pix.0, south_west_pix.1), zoom).unwrap();

    Bounds {
        north_east_lat: north_east_latlng.1,
        north_east_lng: north_east_latlng.0,
        south_west_lat: south_west_latlng.1,
        south_west_lng: south_west_latlng.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static SAMPLE_POINT: Point = Point { lat: 43.0, lng: -79.0, price: 1 };
    static DEFAULT_ZOOM: usize = 8;
    static DEFAULT_BOUNDS: Bounds = Bounds {
        north_east_lat: 45.0,
        north_east_lng: -75.0,
        south_west_lat: 40.0,
        south_west_lng: -81.0,
    };

    #[test]
    fn clusters_include_all_points() {
        let sample_points = vec![ Point { lat: 43.0, lng: -79.0, price: 1 }; 5 ];

        let clustered = cluster_points(&sample_points, &DEFAULT_BOUNDS, DEFAULT_ZOOM);
        let cluster_point_count = clustered.iter().fold(0, |sum, ref x| sum + x.count );
        assert_eq!(sample_points.len() as u32, cluster_point_count);
    }

    #[test]
    fn add_some_points_to_a_cluster() {
        let mut sample_clusters: Vec<Cluster> = Vec::new();

        add_to_closest_cluster(&mut sample_clusters, &SAMPLE_POINT, DEFAULT_ZOOM);

        assert_eq!(sample_clusters.len(), 1);
        assert_eq!(sample_clusters[0].count, 1);
        assert_eq!(sample_clusters[0].center_lat, SAMPLE_POINT.lat);
        assert_eq!(sample_clusters[0].center_lng, SAMPLE_POINT.lng);

        add_to_closest_cluster(&mut sample_clusters, &SAMPLE_POINT, DEFAULT_ZOOM);

        assert_eq!(sample_clusters.len(), 1);
        assert_eq!(sample_clusters[0].count, 2);
        assert_eq!(sample_clusters[0].center_lat, SAMPLE_POINT.lat);
        assert_eq!(sample_clusters[0].center_lng, SAMPLE_POINT.lng);
    }

    #[test]
    fn test_100000_points() {
        let sample_points = vec![ Point { lat: 43.0, lng: -79.0, price: 3 }; 100000 ];
        
        let clustered = cluster_points(&sample_points, &DEFAULT_BOUNDS, DEFAULT_ZOOM);
        assert_eq!(clustered.len(), 1);
        assert_eq!(clustered.get(0).unwrap().count, 100000);
    }

    #[test]
    fn bounds_get_extended() {
        let bounds = Bounds {
            north_east_lat: 43.6532,
            north_east_lng: -79.3832,
            south_west_lat: 43.6532,
            south_west_lng: -79.3832,
        };

        let extended_bounds = calculate_extended_bounds(&bounds, DEFAULT_ZOOM);

        assert!(bounds.north_east_lat < extended_bounds.north_east_lat);
        assert!(bounds.north_east_lng < extended_bounds.north_east_lng);
        assert!(bounds.south_west_lat > extended_bounds.south_west_lat);
        assert!(bounds.south_west_lng > extended_bounds.south_west_lng);
    }

    // zoom level is (index + 3) 
    static GMAP_BOUNDS: [Bounds; 17] = [
        Bounds {north_east_lat: 50.800061065188856, north_east_lng: -68.83632499999999, south_west_lat: 35.542543366259075, south_west_lng: -89.93007499999999},
        Bounds {north_east_lat: 47.34741387849921, north_east_lng: -74.10976249999999, south_west_lat: 39.71693995491094, south_west_lng: -84.65663749999999},
        Bounds {north_east_lat: 45.530626397270055, north_east_lng: -76.74648124999999, south_west_lat: 41.71519339348616, south_west_lng: -82.01991874999999},
        Bounds {north_east_lat: 44.599495541698985, north_east_lng: -78.06484062499999, south_west_lat: 42.69175511293576, south_west_lng: -80.70155937499999},
        Bounds {north_east_lat: 44.12824279122392, north_east_lng: -78.72402031249999, south_west_lat: 43.17436960409823, south_west_lng: -80.04237968749999},
        Bounds {north_east_lat: 43.891195023324286, north_east_lng: -79.05361015624999, south_west_lat: 43.414258058734866, south_west_lng: -79.71278984374999},
        Bounds {north_east_lat: 43.77231589906095, north_east_lng: -79.21840507812499, south_west_lat: 43.5338473704056, south_west_lng: -79.54799492187499},
        Bounds {north_east_lat: 43.712787543711634, north_east_lng: -79.30080253906249, south_west_lat: 43.59355327358944, south_west_lng: -79.46559746093749},
        Bounds {north_east_lat: 43.68300117005328, north_east_lng: -79.34200126953124, south_west_lat: 43.623384034267886, south_west_lng: -79.42439873046874},
        Bounds {north_east_lat: 43.66810243453164, north_east_lng: -79.36260063476561, south_west_lat: 43.63829386654838, south_west_lng: -79.40379936523436},
        Bounds {north_east_lat: 43.66065167963645, north_east_lng: -79.3729003173828, south_west_lat: 43.64574739563353, south_west_lng: -79.39349968261718},
        Bounds {north_east_lat: 43.65692595541019, north_east_lng: -79.3780501586914, south_west_lat: 43.6494738134073, south_west_lng: -79.38834984130858},
        Bounds {north_east_lat: 43.65506300660299, north_east_lng: -79.38062507934569, south_west_lat: 43.65133693560138, south_west_lng: -79.38577492065428},
        Bounds {north_east_lat: 43.65413151052596, north_east_lng: -79.38191253967284, south_west_lat: 43.652268475025124, south_west_lng: -79.38448746032714},
        Bounds {north_east_lat: 43.653665757069085, north_east_lng: -79.38255626983641, south_west_lat: 43.652734239318676, south_west_lng: -79.38384373016356},
        Bounds {north_east_lat: 43.653432878986074, north_east_lng: -79.3828781349182, south_west_lat: 43.65296712011086, south_west_lng: -79.38352186508178},
        Bounds {north_east_lat: 43.6533164396059, north_east_lng: -79.3830390674591, south_west_lat: 43.653083560168305, south_west_lng: -79.38336093254088}
    ];

    #[test]
    fn compare_bound_extension_to_gmap() {
        let bounds = Bounds {
            north_east_lat: 43.6532,
            north_east_lng: -79.3832,
            south_west_lat: 43.6532,
            south_west_lng: -79.3832,
        };

        for (i, g_bounds) in GMAP_BOUNDS.iter().enumerate() {
            let zoom = i + 3;
            let extended_bounds = calculate_extended_bounds(&bounds, zoom);
            bounds_error(&g_bounds, &extended_bounds, zoom);
        }
    }

    fn bounds_error(gmap: &Bounds, wasm: &Bounds, zoom: usize) {
        print!("z{:02} errors: | ", zoom);
        print!("NE lat: {:9.6} | ", percent_error(gmap.north_east_lat, wasm.north_east_lat));
        print!("NE lng: {:9.6} | ", percent_error(gmap.north_east_lng, wasm.north_east_lng));
        print!("SW lat: {:9.6} | ", percent_error(gmap.south_west_lat, wasm.south_west_lat));
        print!("SW lng: {:9.6} | ", percent_error(gmap.south_west_lng, wasm.south_west_lng));
        println!();
    }

    fn percent_error(expected: f64, achieved: f64) -> f64 {
        (achieved - expected)/expected.abs() * 100.0
    }
}