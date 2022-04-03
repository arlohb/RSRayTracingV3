use std::thread;

use rs_ray_tracing_v2::gpu::Gpu;

fn render_thread() {
  loop {
    rs_ray_tracing_v2::ray_tracer::render_image();
  }
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
  env_logger::init();
  let gpu = pollster::block_on(Gpu::new());
  println!("Result: {:?}", pollster::block_on(gpu.run(&[1., 2.])));
  println!("Result: {:?}", pollster::block_on(gpu.run(&[1., 2.])));
  println!("Result: {:?}", pollster::block_on(gpu.run(&[1., 2.])));

  thread::spawn(render_thread);

  let app = rs_ray_tracing_v2::App::new(400, 300);
  let native_options = eframe::NativeOptions {
    initial_window_size: Some(eframe::epaint::Vec2 { x: 1000., y: 800. }),
    ..eframe::NativeOptions::default()
  };
  eframe::run_native(Box::new(app), native_options);
}
