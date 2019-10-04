use uuid::Uuid;
use Marker;

#[derive(Clone, Debug, Serialize)]
pub struct UniqueMarker {
  pub lat: f64,
  pub lng: f64,

  #[serde(skip)]
  uuid: Uuid,

  #[serde(skip)]
  pub is_added: bool,
}

impl From<&Marker> for UniqueMarker {
  fn from(point: &Marker) -> Self {
    UniqueMarker {
      lat: point.lat,
      lng: point.lng,
      uuid: Uuid::new_v4(),
      is_added: false,
    }
  }
}

impl PartialEq for UniqueMarker {
  fn eq(&self, other: &UniqueMarker) -> bool {
    self.uuid == other.uuid
  }
}
