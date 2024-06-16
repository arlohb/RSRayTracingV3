#![warn(
    missing_docs,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::pedantic,
    clippy::nursery,
    future_incompatible
)]
#![allow(
    // I understand how rust modules work,
    // so if I do this its on purpose.
    clippy::module_inception,
    // Same as above.
    clippy::module_name_repetitions,
    // Rust makes it obvious when this is happening
    // without this lint.
    clippy::cast_precision_loss,
    // Same as above.
    clippy::cast_possible_truncation,
    // Same as above.
    clippy::cast_sign_loss,
)]

//! A GPU ray tracer written in Rust.

mod ui;
use app::App;
pub use ui::Ui;

mod app;
mod bytes;
mod gpu;
mod movement;
mod panels;
mod ray_tracer;
mod time;

use anyhow::Result;

#[allow(missing_docs, clippy::missing_errors_doc)]
pub fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    {
        puffin::set_scopes_on(true);
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    App::run()
}
