mod app;
pub use app::App;

pub mod ray_tracer;
pub mod movement;
pub mod panels;
pub mod gpu;

use std::sync::Mutex;
use once_cell::sync::Lazy;

// these can be imported without crate::,
// but I'm doing this to be consistent with the rest of the code
use crate::ray_tracer::Options;
use crate::gpu::Gpu;

static OPTIONS: Lazy<Mutex<Options>> = Lazy::new(||
  Mutex::new(Options::new(400, 300))
);
static IMAGE: Lazy<Mutex<eframe::epaint::image::ColorImage>> = Lazy::new(||
  Mutex::new(eframe::epaint::image::ColorImage::new([400, 300], eframe::epaint::Color32::BLACK))
);
static FRAME_TIMES: Lazy<Mutex<eframe::egui::util::History<f32>>> = Lazy::new(||
  Mutex::new(eframe::egui::util::History::new(0..usize::MAX, 1_000.)) // 1 second
);

struct Time {}

impl Time {
  pub fn now() -> f64 {
    std::time::SystemTime::now()
      .duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap()
      .as_micros() as f64 / 1000.
  }
}


fn render_thread() {
  loop {
    crate::ray_tracer::render_image();
  }
}

fn main() {
  env_logger::init();
  let gpu = pollster::block_on(Gpu::new());
  println!("Result: {:?}", pollster::block_on(gpu.run(&[1., 2.])));
  println!("Result: {:?}", pollster::block_on(gpu.run(&[1., 2.])));
  println!("Result: {:?}", pollster::block_on(gpu.run(&[1., 2.])));

  std::thread::spawn(render_thread);

  let app = crate::App::new(400, 300);
  let native_options = eframe::NativeOptions {
    initial_window_size: Some(eframe::epaint::Vec2 { x: 1000., y: 800. }),
    ..eframe::NativeOptions::default()
  };
  eframe::run_native(Box::new(app), native_options);
}
