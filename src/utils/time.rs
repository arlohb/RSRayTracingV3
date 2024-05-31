#[cfg(not(target_arch = "wasm32"))]
use std::time::SystemTime;

#[cfg(not(target_arch = "wasm32"))]
pub fn now_millis() -> f64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_micros() as f64
        / 1000.
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Date, js_name = now)]
    fn date_now() -> f64;
}

#[cfg(target_arch = "wasm32")]
pub fn now_millis() -> f64 {
    date_now()
}
