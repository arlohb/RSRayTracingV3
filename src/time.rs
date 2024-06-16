use anyhow::Result;
use std::time::SystemTime;

/// Get the milliseconds since the unix epoch.
///
/// # Errors
///
/// If the current time is somehow before the unix epoch.
pub fn now_millis() -> Result<f64> {
    Ok(SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_micros() as f64
        / 1000.)
}
