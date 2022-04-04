mod app;
pub use app::App;

pub mod ray_tracer;
pub mod movement;
pub mod panels;
pub mod gpu;

use std::sync::{Mutex, Arc};

// these can be imported without crate::,
// but I'm doing this to be consistent with the rest of the code
use crate::ray_tracer::Options;
use crate::gpu::Gpu;

struct Time {}

impl Time {
  pub fn now() -> f64 {
    std::time::SystemTime::now()
      .duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap()
      .as_micros() as f64 / 1000.
  }
}

fn main() {
  rayon::ThreadPoolBuilder::new()
    .num_threads(num_cpus::get())
    .build_global()
    .unwrap();

  let g_renderer = Arc::new(Mutex::new(Options::new(400, 300)));
  let image = Arc::new(Mutex::new(eframe::epaint::image::ColorImage::new([400, 300], eframe::epaint::Color32::BLACK)));
  let frame_times = Arc::new(Mutex::new(eframe::egui::util::History::<f32>::new(0..usize::MAX, 1_000.))); // 1 second

  env_logger::init();
  let gpu = pollster::block_on(Gpu::new());
  println!("Result: {:?}", pollster::block_on(gpu.run(&[1., 2.])));
  println!("Result: {:?}", pollster::block_on(gpu.run(&[1., 2.])));
  println!("Result: {:?}", pollster::block_on(gpu.run(&[1., 2.])));

  let app = crate::App::new(400, 300, g_renderer.clone(), image.clone(), frame_times.clone());
  
  std::thread::spawn(move || loop {
    crate::ray_tracer::render_image(g_renderer.clone(), image.clone(), frame_times.clone());
  });

  let native_options = eframe::NativeOptions {
    initial_window_size: Some(eframe::epaint::Vec2 { x: 1000., y: 800. }),
    ..eframe::NativeOptions::default()
  };
  eframe::run_native(Box::new(app), native_options);
}
