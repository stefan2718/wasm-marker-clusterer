use uuid::Uuid;
use Marker;
use UniqueMarker;
use Bounds;

#[derive(Debug, Serialize, Clone)]
pub struct Cluster {
    pub uuid: Uuid,
    pub size: u32,
    pub center: Marker,
    pub markers: Vec<UniqueMarker>,
    pub bounds: Bounds,
}

impl Cluster {
    pub fn add_marker(&mut self, new_point: &UniqueMarker, zoom: usize, average_center: bool, grid_size: f64) {
        self.size += 1;
        self.markers.push(new_point.clone());
        if average_center {
            self.center.lat = ((self.center.lat * f64::from(self.size)) + new_point.lat) / f64::from(self.size + 1);
            self.center.lng = ((self.center.lng * f64::from(self.size)) + new_point.lng) / f64::from(self.size + 1);
            self.bounds = Bounds::from_point(self.center.lat, self.center.lng, zoom, grid_size);
        }
    }
}