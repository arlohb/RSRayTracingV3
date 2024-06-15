use anyhow::Result;
use std::time::SystemTime;

pub fn now_millis() -> Result<f64> {
    Ok(SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_micros() as f64
        / 1000.)
}
