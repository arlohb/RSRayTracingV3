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
pub use ui::Ui;

mod app;
mod bytes;
mod gpu;
mod movement;
mod panels;
mod ray_tracer;
mod time;

use anyhow::Result;
use std::sync::Arc;

use ray_tracer::Scene;

#[allow(missing_docs, clippy::missing_errors_doc)]
pub fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    {
        puffin::set_scopes_on(true);
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    // TODO: Move all this into App
    let scene = Scene::random_spheres_default_config();

    let initial_window_size = (1920u32, 1080u32);
    let initial_render_size = (1000u32, 900u32);

    let event_loop = egui_winit::winit::event_loop::EventLoop::new()?;

    let window = Arc::new(
        egui_winit::winit::window::WindowBuilder::new()
            .with_decorations(true)
            .with_resizable(true)
            .with_transparent(false)
            .with_title("Ray Tracer")
            .with_inner_size(egui_winit::winit::dpi::PhysicalSize {
                width: initial_window_size.0,
                height: initial_window_size.1,
            })
            .build(&event_loop)?,
    );

    app::run(event_loop, window, scene, initial_render_size)
}
