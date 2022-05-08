pub mod bytes;
pub mod history;
pub mod time;

#[cfg(not(target_arch = "wasm32"))]
macro_rules! log {
  ( $( $t:tt )* ) => {
    println!( $( $t )* );
  }
}

#[cfg(target_arch = "wasm32")]
macro_rules! log {
  ( $( $t:tt )* ) => {
    web_sys::console::log_1(&format!( $( $t )* ).into());
  }
}

pub(crate) use log;
