extern crate wasm_bindgen;
extern crate web_sys;

#[macro_use]
extern crate serde_derive;

mod utils;

use wasm_bindgen::prelude::*;
use web_sys::console;
use std::f64;

// Cluster struct
// - Should maintain list of points in the cluster, but not return that list to JS
#[derive(Debug, Serialize)]
pub struct Cluster {
    count: u32,
    center_lat: f64,
    center_lng: f64,
    // points: Vec<&Point>,
}

#[wasm_bindgen]
impl Cluster {
    // #[wasm_bindgen(constructor)]
    // pub fn new (lat: f64, lng: f64) -> Cluster {
    //     Cluster {
    //         count: 1,
    //         center_lat: lat,
    //         center_lng: lng,
    //     }
    // }

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
pub fn parse_and_cluster_points(points_val: &JsValue) -> JsValue {
    utils::set_panic_hook();

    let points: Vec<Point> = points_val.into_serde().unwrap();
    console::time_end_with_label("into-wasm");
    console::time_with_label("clustering");
    let clusters = cluster_points(&points);
    console::time_end_with_label("clustering");
    console::time_with_label("out-of-wasm");
    return JsValue::from_serde(&clusters).unwrap();
}

pub fn cluster_points(points: &Vec<Point>) -> Vec<Cluster> {
    let mut clusters = Vec::new();
    for point in points.iter() {
        add_to_closest_cluster(&mut clusters, point);
    }
    clusters
}

pub fn add_to_closest_cluster(clusters: &mut Vec<Cluster>, new_point: &Point) {
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

    match cluster_index_to_add_to {
        Some(index) => clusters[index].add_marker(new_point),
        None => clusters.push(Cluster {
            count: 1,
            center_lat: new_point.lat,
            center_lng: new_point.lng
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clusters_include_all_points() {
        let sample_points = vec![ Point { lat: 1.0, lng: 1.0, price: 1 }; 5 ];

        let clustered = cluster_points(&sample_points);
        let cluster_point_count = clustered.iter().fold(0, |sum, ref x| sum + x.count );
        assert_eq!(sample_points.len() as u32, cluster_point_count);
    }

    #[test]
    fn add_some_points_to_a_cluster() {
        let mut sample_clusters: Vec<Cluster> = Vec::new();

        add_to_closest_cluster(&mut sample_clusters, &Point { lat: 1.0, lng: 1.0, price: 1 });

        assert_eq!(sample_clusters.len(), 1);
        assert_eq!(sample_clusters[0].count, 1);
        assert_eq!(sample_clusters[0].center_lat, 1.0);
        assert_eq!(sample_clusters[0].center_lng, 1.0);

        add_to_closest_cluster(&mut sample_clusters, &Point { lat: 2.0, lng: 2.0, price: 2 });

        assert_eq!(sample_clusters.len(), 1);
        assert_eq!(sample_clusters[0].count, 2);
        assert_eq!(sample_clusters[0].center_lat, 1.5);
        assert_eq!(sample_clusters[0].center_lng, 1.5);
    }

    #[test]
    fn test_100000_points() {
        let sample_points = vec![ Point { lat: 1.0, lng: 2.0, price: 3 }; 100000 ];
        
        let clustered = cluster_points(&sample_points);
        assert_eq!(clustered.len(), 1);
        assert_eq!(clustered.get(0).unwrap().count, 100000);
    }
}