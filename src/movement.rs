use eframe::egui;

use crate::ray_tracer::Renderer;

pub fn move_and_rotate(
  input: &egui::InputState,
  ray_tracer: &mut Renderer,
  look_speed: f64,
  move_speed: f64
) {
  if input.key_down(egui::Key::ArrowRight) {
    ray_tracer.rotation.y += look_speed;
  }
  if input.key_down(egui::Key::ArrowLeft) {
    ray_tracer.rotation.y -= look_speed;
  }
  if input.key_down(egui::Key::ArrowUp) {
    ray_tracer.rotation.x -= look_speed;
  }
  if input.key_down(egui::Key::ArrowDown) {
    ray_tracer.rotation.x += look_speed;
  }

  ray_tracer.rotation.x = ray_tracer.rotation.x.clamp(-0.5 * std::f64::consts::PI, 0.5 * std::f64::consts::PI);
  ray_tracer.rotation.y %= 2. * std::f64::consts::PI;
  ray_tracer.rotation.z = ray_tracer.rotation.z.clamp(-std::f64::consts::PI, std::f64::consts::PI);

  if input.key_down(egui::Key::W) {
    ray_tracer.camera -= ray_tracer.forward() * move_speed;
  }
  if input.key_down(egui::Key::S) {
    ray_tracer.camera += ray_tracer.forward() * move_speed;
  }
  if input.key_down(egui::Key::D) {
    ray_tracer.camera += ray_tracer.right() * move_speed;
  }
  if input.key_down(egui::Key::A) {
    ray_tracer.camera -= ray_tracer.right() * move_speed;
  }
  if input.key_down(egui::Key::E) {
    ray_tracer.camera += ray_tracer.up() * move_speed;
  }
  if input.key_down(egui::Key::Q) {
    ray_tracer.camera -= ray_tracer.up() * move_speed;
  }
}