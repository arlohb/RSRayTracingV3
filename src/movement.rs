use eframe::egui;

use crate::ray_tracer::Camera;

pub fn move_and_rotate(
  input: &egui::InputState,
  camera: &mut Camera,
  look_speed: f64,
  move_speed: f64,
  shift_mod: f64,
  ctrl_mod: f64,
) {
  if input.key_down(egui::Key::ArrowRight) {
    camera.rotation.y += look_speed;
  }
  if input.key_down(egui::Key::ArrowLeft) {
    camera.rotation.y -= look_speed;
  }
  if input.key_down(egui::Key::ArrowUp) {
    camera.rotation.x -= look_speed;
  }
  if input.key_down(egui::Key::ArrowDown) {
    camera.rotation.x += look_speed;
  }

  // x should be clamped between -pi/2 and pi/2
  camera.rotation.x = camera.rotation.x.clamp(-0.5 * std::f64::consts::PI, 0.5 * std::f64::consts::PI);

  // y should be wrapped around to between -pi and pi
  if camera.rotation.y < std::f64::consts::PI { camera.rotation.y += 2. * std::f64::consts::PI; }
  if camera.rotation.y > std::f64::consts::PI { camera.rotation.y -= 2. * std::f64::consts::PI; }

  // z should be clamped between -pi and pi
  camera.rotation.z = camera.rotation.z.clamp(-std::f64::consts::PI, std::f64::consts::PI);

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
    camera.position -= camera.forward() * move_speed;
  }
  if input.key_down(egui::Key::S) {
    camera.position += camera.forward() * move_speed;
  }
  if input.key_down(egui::Key::D) {
    camera.position += camera.right() * move_speed;
  }
  if input.key_down(egui::Key::A) {
    camera.position -= camera.right() * move_speed;
  }
  if input.key_down(egui::Key::E) {
    camera.position += camera.up() * move_speed;
  }
  if input.key_down(egui::Key::Q) {
    camera.position -= camera.up() * move_speed;
  }
}