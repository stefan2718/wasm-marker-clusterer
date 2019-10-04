#[derive(Debug, OptionalStruct)]
#[optional_derive(Deserialize)]
pub struct Config {
  pub grid_size: f64,
  pub average_center: bool,
  pub log_time: bool,
}
