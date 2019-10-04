use UniqueMarker;
use utils::calculate_extended_bounds;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bounds {
  pub north: f64,
  pub east: f64,
  pub south: f64,
  pub west: f64,
}

impl Bounds {
  pub fn contains(&self, point: &UniqueMarker) -> bool {
    self.north > point.lat &&
    self.east > point.lng &&
    self.south < point.lat &&
    self.west < point.lng
  }

  pub fn from_point(lat: f64, lng: f64, zoom: usize, grid_size: f64) -> Bounds {
    calculate_extended_bounds(&Bounds {
      north: lat,
      east: lng,
      south: lat,
      west: lng
    }, zoom, grid_size)
  }
}
