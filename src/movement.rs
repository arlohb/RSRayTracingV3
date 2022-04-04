use eframe::egui::{InputState, Key};

use crate::ray_tracer::Camera;

pub fn move_and_rotate(
  input: &InputState,
  camera: &mut Camera,
  look_speed: f64,
  move_speed: f64,
  shift_mod: f64,
  ctrl_mod: f64,
) {
  if input.key_down(Key::ArrowRight) { camera.rotation.y += look_speed; }
  if input.key_down(Key::ArrowLeft) { camera.rotation.y -= look_speed; }
  if input.key_down(Key::ArrowUp) { camera.rotation.x -= look_speed; }
  if input.key_down(Key::ArrowDown) { camera.rotation.x += look_speed; }

  camera.clamp_rotation();

  let move_speed = if input.modifiers.shift {
    move_speed * shift_mod
  } else if input.modifiers.ctrl {
    move_speed * ctrl_mod
  } else {
    move_speed
  };

  if input.key_down(Key::W) { camera.position -= camera.forward() * move_speed; }
  if input.key_down(Key::S) { camera.position += camera.forward() * move_speed; }
  if input.key_down(Key::D) { camera.position += camera.right() * move_speed; }
  if input.key_down(Key::A) { camera.position -= camera.right() * move_speed; }
  if input.key_down(Key::E) { camera.position += camera.up() * move_speed; }
  if input.key_down(Key::Q) { camera.position -= camera.up() * move_speed; }
}