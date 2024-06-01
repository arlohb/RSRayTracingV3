mod ui;
pub use ui::Ui;

pub mod app;
pub mod gpu;
pub mod movement;
pub mod panels;
pub mod ray_tracer;
pub mod utils;

use std::sync::{Arc, Mutex};

use ray_tracer::*;

pub fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    let frame_times = Arc::new(Mutex::new(utils::history::History::new(5_000.)));
    let scene = Arc::new(Mutex::new(Scene::random_sphere_default_config()));

    let fps_limit = 5000.;
    let initial_window_size = (1920u32, 1080u32);
    let initial_render_size = (1000u32, 900u32);

    let event_loop = winit::event_loop::EventLoop::new();

    let window = winit::window::WindowBuilder::new()
        .with_decorations(true)
        .with_resizable(true)
        .with_transparent(false)
        .with_title("Ray Tracer")
        .with_inner_size(winit::dpi::PhysicalSize {
            width: initial_window_size.0,
            height: initial_window_size.1,
        })
        .build(&event_loop)
        .expect("Failed to create window");

    pollster::block_on(crate::app::run(
        event_loop,
        window,
        scene,
        frame_times,
        fps_limit,
        initial_render_size,
    ));
}
