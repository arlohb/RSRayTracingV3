mod app;
pub use app::App;

pub mod ray_tracer;
pub mod movement;
pub mod panels;
pub mod gpu;
pub mod time;
pub mod utils;
pub mod history;

pub use time::Time;
pub use history::History;

use std::sync::{Mutex, Arc};
use winit::platform::unix::EventLoopExtUnix;

use ray_tracer::*;

fn main() {
  let frame_times = Arc::new(Mutex::new(History::new(5_000.)));
  let scene = Arc::new(Mutex::new(Scene::random_sphere_default_config()));

  let app = Box::new(
    crate::App::new(scene.clone(), frame_times.clone()),
  );

  std::thread::spawn(move || {
    let event_loop = winit::event_loop::EventLoop::new_any_thread();
    let window = winit::window::Window::new(&event_loop).unwrap();
    pollster::block_on(crate::gpu::run(event_loop, window, scene, frame_times, 240.));
  });

  eframe::run_native(
    app,
    eframe::NativeOptions {
      initial_window_size: Some(eframe::epaint::Vec2 { x: 700., y: 800. }),
      ..eframe::NativeOptions::default()
    },
  );
}
