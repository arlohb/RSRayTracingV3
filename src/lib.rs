#![warn(
    // TODO: Enable this
    // missing_docs,
    clippy::unwrap_used,
    // TODO: Improve error handling
    // clippy::expect_used,
    clippy::pedantic,
    clippy::nursery,
    future_incompatible,
)]
#![allow(
    // TODO: Improve error handling
    clippy::missing_panics_doc,
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

mod ui;
pub use ui::Ui;

pub mod app;
pub mod gpu;
pub mod movement;
pub mod panels;
pub mod ray_tracer;
pub mod utils;

use std::sync::Arc;

use ray_tracer::Scene;

pub fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    #[cfg(debug_assertions)]
    puffin::set_scopes_on(true);

    let scene = Scene::random_spheres_default_config();

    let initial_window_size = (1920u32, 1080u32);
    let initial_render_size = (1000u32, 900u32);

    let event_loop =
        egui_winit::winit::event_loop::EventLoop::new().expect("Failed to create event loop");

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
            .build(&event_loop)
            .expect("Failed to create window"),
    );

    app::run(event_loop, window, scene, initial_render_size);
}
