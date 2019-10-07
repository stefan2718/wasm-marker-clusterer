#[derive(Debug, OptionalStruct)]
#[optional_derive(Deserialize)]
pub struct Config {
  pub grid_size: f64,
  pub average_center: bool,
  pub log_time: bool,
  pub only_return_modified_clusters: bool,
}

impl Default for Config {
  fn default() -> Config {
    Config {
      grid_size: 60.0,
      average_center: false,
      log_time: false,
      only_return_modified_clusters: true,
    }
  }
}