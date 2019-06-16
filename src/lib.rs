extern crate wasm_bindgen;
#[macro_use]
extern crate serde_derive;

mod utils;

use wasm_bindgen::prelude::*;

// Cluster struct
// - Should maintain list of points in the cluster, but not return that list to JS
#[derive(Debug, Serialize)]
pub struct Cluster {
    count: u16,
    center_x: f64,
    center_y: f64,
    // points: Vec<&Point>,
}

#[wasm_bindgen]
impl Cluster {
    // #[wasm_bindgen(constructor)]
    // pub fn new (x: f64, y: f64) -> Cluster {
    //     Cluster {
    //         count: 1,
    //         center_x: x,
    //         center_y: y,
    //     }
    // }
}

#[derive(Debug, Deserialize)]
pub struct Point {
    x: f64,
    y: f64,
    price: u32,
}

#[wasm_bindgen]
impl Point {

}

#[wasm_bindgen]
pub fn parse_and_cluster_points(points_val: &JsValue) -> JsValue {
    utils::set_panic_hook();

    let points: Vec<Point> = points_val.into_serde().unwrap();
    let clusters = cluster_points(&points);
    return JsValue::from_serde(&clusters).unwrap();
}

pub fn cluster_points(points: &Vec<Point>) -> Vec<Cluster> {
    let mut v = Vec::new();
    for point in points.iter() {
        v.push(Cluster {
            count: 1,
            center_x: point.x, 
            center_y: point.y,
            // points: vec![point]
        });
    }
    return v;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clusters_include_all_points() {
        let sample_points= vec![
            Point { x: 1.0, y: 1.0, price: 1 },
            Point { x: 1.0, y: 1.0, price: 1 },
            Point { x: 1.0, y: 1.0, price: 1 },
            Point { x: 1.0, y: 1.0, price: 1 },
            Point { x: 1.0, y: 1.0, price: 1 },
        ];

        let clustered = cluster_points(&sample_points);
        let cluster_point_count = clustered.iter().fold(0, |sum, ref x| sum + x.count );
        assert_eq!(sample_points.len() as u16, cluster_point_count);
    }
}