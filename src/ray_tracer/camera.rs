use serde::{Deserialize, Serialize};
use crate::ray_tracer::*;

#[derive(Deserialize, Serialize, Clone)]
pub struct Camera {
  pub position: Vec3,
  // in radians
  pub rotation: Vec3,
  pub fov: f64,
}

impl Camera {
  pub fn clamp_rotation(&mut self) {
    // x should be clamped between -pi/2 and pi/2
    self.rotation.x = self.rotation.x.clamp(-0.5 * std::f64::consts::PI, 0.5 * std::f64::consts::PI);

    // y should be wrapped around to between -pi and pi
    if self.rotation.y < std::f64::consts::PI { self.rotation.y += 2. * std::f64::consts::PI; }
    if self.rotation.y > std::f64::consts::PI { self.rotation.y -= 2. * std::f64::consts::PI; }

    // z should be clamped between -pi and pi
    self.rotation.z = self.rotation.z.clamp(-std::f64::consts::PI, std::f64::consts::PI);
  }

  pub fn forward(&self) -> Vec3 {
    Vec3 { x: 0., y: 0., z: 1. }
      .transform_point(Mat44::create_rotation(Axis::X, -self.rotation.x))
      .transform_point(Mat44::create_rotation(Axis::Y, -self.rotation.y))
  }

  pub fn right(&self) -> Vec3 {
    let temp = Vec3 { x: 0., y: 1., z: 0. }
      .transform_point(Mat44::create_rotation(Axis::Z, -self.rotation.z));
    (temp * self.forward()).normalize()
  }

  pub fn up(&self) -> Vec3 {
    (self.forward() * self.right()).normalize()
  }
}
