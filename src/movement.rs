use eframe::egui;

use crate::ray_tracer::Vec3;

pub fn move_and_rotate(
  input: &egui::InputState,
  position: &mut Vec3,
  rotation: &mut Vec3,
  forward: Vec3,
  right: Vec3,
  up: Vec3,
  look_speed: f64,
  move_speed: f64
) {
  if input.key_down(egui::Key::ArrowRight) {
    rotation.y += look_speed;
  }
  if input.key_down(egui::Key::ArrowLeft) {
    rotation.y -= look_speed;
  }
  if input.key_down(egui::Key::ArrowUp) {
    rotation.x -= look_speed;
  }
  if input.key_down(egui::Key::ArrowDown) {
    rotation.x += look_speed;
  }

  rotation.x = rotation.x.clamp(-0.5 * std::f64::consts::PI, 0.5 * std::f64::consts::PI);
  rotation.y %= 2. * std::f64::consts::PI;
  rotation.z = rotation.z.clamp(-std::f64::consts::PI, std::f64::consts::PI);

  if input.key_down(egui::Key::W) {
    *position -= forward * move_speed;
  }
  if input.key_down(egui::Key::S) {
    *position += forward * move_speed;
  }
  if input.key_down(egui::Key::D) {
    *position += right * move_speed;
  }
  if input.key_down(egui::Key::A) {
    *position -= right * move_speed;
  }
  if input.key_down(egui::Key::E) {
    *position += up * move_speed;
  }
  if input.key_down(egui::Key::Q) {
    *position -= up * move_speed;
  }
}