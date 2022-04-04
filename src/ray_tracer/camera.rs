use serde::{Deserialize, Serialize};
use crate::ray_tracer::*;

#[derive(Deserialize, Serialize, Clone)]
pub struct Camera {
  pub position: Vec3,
  pub rotation: Vec3,
  pub fov: f64,
}

impl Camera {
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
