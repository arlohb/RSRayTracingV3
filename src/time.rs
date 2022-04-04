use std::time::SystemTime;

pub struct Time {}

impl Time {
  pub fn now() -> f64 {
    SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH).unwrap()
      .as_micros() as f64 / 1000.
  }
}
