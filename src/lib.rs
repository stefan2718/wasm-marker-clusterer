extern crate wasm_bindgen;

// mod utils;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

// Cluster struct
// - Should maintain list of points in the cluster, but not return that list to JS
#[wasm_bindgen]
#[derive(Debug)]
pub struct Cluster {
    count: u16,
    center: [f64; 2],
    // points: Vec<&Point>,
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct Point {
    x: f64,
    y: f64,
    price: u32,
}

#[wasm_bindgen]
pub fn cluster_points(points: &[Point]) -> Vec<Cluster> {
    let mut v = Vec::new();
    for point in points.iter() {
        v.push(Cluster {
            count: 1,
            center: [point.x, point.y],
            // points: vec![point]
        });
    }
    return v;
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_POINTS: [Point; 5] = [
        Point { x: 1.0, y: 1.0, price: 1 },
        Point { x: 1.0, y: 1.0, price: 1 },
        Point { x: 1.0, y: 1.0, price: 1 },
        Point { x: 1.0, y: 1.0, price: 1 },
        Point { x: 1.0, y: 1.0, price: 1 },
    ];

    #[test]
    fn clusters_include_all_points() {
        let clustered = cluster_points(&SAMPLE_POINTS);
        let cluster_point_count = clustered.iter().fold(0, |sum, ref x| sum + x.count );
        assert_eq!(SAMPLE_POINTS.len() as u16, cluster_point_count);
    }
}