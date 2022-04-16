mod ui;
pub use ui::Ui;

pub mod ray_tracer;
pub mod movement;
pub mod panels;
pub mod gpu;
pub mod utils;
pub mod app;

use std::sync::{Mutex, Arc};

use ray_tracer::*;

fn main() {
  std::env::set_var("RUST_BACKTRACE", "1");

  let frame_times = Arc::new(Mutex::new(utils::history::History::new(5_000.)));
  let scene = Arc::new(Mutex::new(Scene::random_sphere_default_config()));

  crate::app::run(
    scene,
    frame_times,
    60.,
    (1920, 1080),
    (1000, 900),
  );
}
