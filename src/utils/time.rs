use std::time::SystemTime;

pub fn now_millis() -> f64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_micros() as f64
        / 1000.
}
