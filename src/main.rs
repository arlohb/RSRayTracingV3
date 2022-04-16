mod app;
pub use app::App;

pub mod ray_tracer;
pub mod movement;
pub mod panels;
pub mod gpu;
pub mod time;
pub mod utils;
pub mod history;
pub mod wgpu_app;

pub use time::Time;
pub use history::History;

use std::sync::{Mutex, Arc};

use ray_tracer::*;

fn main() {
  let frame_times = Arc::new(Mutex::new(History::new(5_000.)));
  let scene = Arc::new(Mutex::new(Scene::random_sphere_default_config()));

  pollster::block_on(crate::wgpu_app::run(scene, frame_times, 240.));
}
