mod app;
pub use app::App;

pub mod ray_tracer;
pub mod movement;
pub mod panels;
pub mod gpu;
pub mod time;

// I want to use this across the project without importing it
pub use time::Time;

use std::sync::{Mutex, Arc};
use winit::platform::unix::EventLoopExtUnix;

// these can be imported without crate::,
// but I'm doing this to be consistent with the rest of the code
use crate::ray_tracer::Renderer;
use crate::gpu::Gpu;

fn main() {
  // create the global thread pool
  rayon::ThreadPoolBuilder::new()
    .num_threads(num_cpus::get())
    .build_global()
    .expect("Failed to create thread pool");

  let renderer = Arc::new(Mutex::new(Renderer::new(400, 300)));
  let image = Arc::new(Mutex::new(eframe::epaint::image::ColorImage::new([400, 300], eframe::epaint::Color32::BLACK)));
  let frame_times = Arc::new(Mutex::new(eframe::egui::util::History::<f32>::new(0..usize::MAX, 1_000.))); // 1 second

  std::thread::spawn(|| {
    let event_loop = winit::event_loop::EventLoop::new_any_thread();
    let window = winit::window::Window::new(&event_loop).unwrap();
    pollster::block_on(Gpu::run(event_loop, window));
  });

  // create the renderer thread
  ray_tracer::start_render_thread(renderer.clone(), image.clone(), frame_times.clone());

  eframe::run_native(
    Box::new(
      crate::App::new(renderer, image, frame_times),
    ),
    eframe::NativeOptions {
      initial_window_size: Some(eframe::epaint::Vec2 { x: 1000., y: 800. }),
      ..eframe::NativeOptions::default()
    },
  );
}
