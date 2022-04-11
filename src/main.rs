mod app;
pub use app::App;

pub mod ray_tracer;
pub mod movement;
pub mod panels;
pub mod gpu;
pub mod time;
pub mod utils;

// I want to use this across the project without importing it
pub use time::Time;

use std::sync::{Mutex, Arc};
use winit::platform::unix::EventLoopExtUnix;

use ray_tracer::*;

fn main() {
  // create the global thread pool
  rayon::ThreadPoolBuilder::new()
    .num_threads(num_cpus::get())
    .build_global()
    .expect("Failed to create thread pool");

  let renderer = Arc::new(Mutex::new(Renderer::new(400, 300)));
  let image = Arc::new(Mutex::new(eframe::epaint::image::ColorImage::new([400, 300], eframe::epaint::Color32::BLACK)));
  let frame_times = Arc::new(Mutex::new(eframe::egui::util::History::<f32>::new(0..usize::MAX, 1_000.))); // 1 second
  let scene = Arc::new(Mutex::new(Scene::random_sphere_default_config()));

  std::thread::spawn(move || {
    let event_loop = winit::event_loop::EventLoop::new_any_thread();
    let window = winit::window::Window::new(&event_loop).unwrap();
    pollster::block_on(crate::gpu::run(event_loop, window, scene));
  });

  // create the renderer thread
  ray_tracer::start_render_thread(renderer.clone(), image.clone(), frame_times.clone());

  eframe::run_native(
    Box::new(
      crate::App::new(renderer, frame_times),
    ),
    eframe::NativeOptions {
      initial_window_size: Some(eframe::epaint::Vec2 { x: 600., y: 800. }),
      ..eframe::NativeOptions::default()
    },
  );
}
