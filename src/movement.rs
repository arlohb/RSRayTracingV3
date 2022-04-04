use eframe::egui;

use crate::ray_tracer::Renderer;

pub fn move_and_rotate(
  input: &egui::InputState,
  renderer: &mut Renderer,
  look_speed: f64,
  move_speed: f64,
  shift_mod: f64,
  ctrl_mod: f64,
) {
  if input.key_down(egui::Key::ArrowRight) {
    renderer.rotation.y += look_speed;
  }
  if input.key_down(egui::Key::ArrowLeft) {
    renderer.rotation.y -= look_speed;
  }
  if input.key_down(egui::Key::ArrowUp) {
    renderer.rotation.x -= look_speed;
  }
  if input.key_down(egui::Key::ArrowDown) {
    renderer.rotation.x += look_speed;
  }

  renderer.rotation.x = renderer.rotation.x.clamp(-0.5 * std::f64::consts::PI, 0.5 * std::f64::consts::PI);
  renderer.rotation.y %= 2. * std::f64::consts::PI;
  renderer.rotation.z = renderer.rotation.z.clamp(-std::f64::consts::PI, std::f64::consts::PI);

  let move_speed = if input.modifiers.shift {
    move_speed * shift_mod
  } else {
    move_speed
  };

  let move_speed = if input.modifiers.ctrl {
    move_speed * ctrl_mod
  } else {
    move_speed
  };

  if input.key_down(egui::Key::W) {
    renderer.camera -= renderer.forward() * move_speed;
  }
  if input.key_down(egui::Key::S) {
    renderer.camera += renderer.forward() * move_speed;
  }
  if input.key_down(egui::Key::D) {
    renderer.camera += renderer.right() * move_speed;
  }
  if input.key_down(egui::Key::A) {
    renderer.camera -= renderer.right() * move_speed;
  }
  if input.key_down(egui::Key::E) {
    renderer.camera += renderer.up() * move_speed;
  }
  if input.key_down(egui::Key::Q) {
    renderer.camera -= renderer.up() * move_speed;
  }
}