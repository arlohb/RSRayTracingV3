use std::ops;
use serde::{Deserialize, Serialize};

use crate::ray_tracer::Mat44;

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct Vec3 {
  pub x: f64,
  pub y: f64,
  pub z: f64,
}

impl Vec3 {
  const EPSILON: f64 = 0.000001;
  
  pub fn length(&self) -> f64 {
    ((self.x.powi(2)) + (self.y.powi(2)) + (self.z.powi(2))).sqrt()
  }

  pub fn dot(&self, v: Vec3) -> f64 {
    (self.x * v.x) 
      + (self.y * v.y)
      + (self.z * v.z)
  }

  pub fn normalize(&self) -> Vec3 {
    let length_squared = self.dot(*self);

    if length_squared > 0. {
      let inverse_length = 1. / length_squared.sqrt();
      return Vec3 {
        x: self.x * inverse_length,
        y: self.y * inverse_length,
        z: self.z * inverse_length,
      };
    }

    *self
  }

  pub fn transform_point(&self, mat: Mat44) -> Vec3 {
    Vec3 {
      x: self.x * mat[0][0] + self.y * mat[1][0] + self.z * mat[2][0],
      y: self.x * mat[0][1] + self.y * mat[1][1] + self.z * mat[2][1],
      z: self.x * mat[0][2] + self.y * mat[1][2] + self.z * mat[2][2],
    }
  }
}

impl ops::Add for Vec3 {
  type Output = Vec3;
  fn add(self, rhs: Vec3) -> Vec3 {
    Vec3 {
      x: self.x + rhs.x,
      y: self.y + rhs.y,
      z: self.z + rhs.z,
    }
  }
}

impl ops::AddAssign for Vec3 {
  fn add_assign(&mut self, rhs: Vec3) {
    *self = Vec3 {
      x: self.x + rhs.x,
      y: self.y + rhs.y,
      z: self.z + rhs.z
    }
  }
}

impl ops::Sub for Vec3 {
  type Output = Vec3;
  fn sub(self, rhs: Vec3) -> Vec3 {
    Vec3 {
      x: self.x - rhs.x,
      y: self.y - rhs.y,
      z: self.z - rhs.z,
    }
  }
}

impl ops::SubAssign for Vec3 {
  fn sub_assign(&mut self, rhs: Vec3) {
    *self = Vec3 {
      x: self.x - rhs.x,
      y: self.y - rhs.y,
      z: self.z - rhs.z,
    }
  }
}

impl ops::Neg for Vec3 {
  type Output = Vec3;
  fn neg(self) -> Vec3 {
    Vec3 {
      x: -self.x,
      y: -self.y,
      z: -self.z,
    }
  }
}

impl ops::Mul<Vec3> for Vec3 {
  type Output = Vec3;
  fn mul(self, rhs: Vec3) -> Vec3 {
    Vec3 {
      x: self.y * rhs.z - self.z * rhs.y,
      y: self.z * rhs.x - self.x * rhs.z,
      z: self.x * rhs.y - self.y * rhs.x,
    }
  }
}

impl ops::MulAssign<Vec3> for Vec3 {
  fn mul_assign(&mut self, rhs: Vec3) {
    *self = Vec3 {
      x: self.y * rhs.z - self.z * rhs.y,
      y: self.z * rhs.x - self.x * rhs.z,
      z: self.x * rhs.y - self.y * rhs.x,
    }
  }
}

impl ops::Mul<f64> for Vec3 {
  type Output = Vec3;
  fn mul(self, rhs: f64) -> Vec3 {
    Vec3 {
      x: self.x * rhs,
      y: self.y * rhs,
      z: self.z * rhs,
    }
  }
}

impl ops::MulAssign<f64> for Vec3 {
  fn mul_assign(&mut self, rhs: f64) {
    *self = Vec3 {
      x: self.x * rhs,
      y: self.y * rhs,
      z: self.z * rhs,
    }
  }
}

impl ops::Div<f64> for Vec3 {
  type Output = Vec3;
  fn div(self, rhs: f64) -> Vec3 {
    Vec3 {
      x: self.x / rhs,
      y: self.y / rhs,
      z: self.z / rhs,
    }
  }
}

impl ops::DivAssign<f64> for Vec3 {
  fn div_assign(&mut self, rhs: f64) {
    *self = Vec3 {
      x: self.x / rhs,
      y: self.y / rhs,
      z: self.z / rhs,
    }
  }
}

impl PartialEq for Vec3 {
  fn eq(&self, other: &Vec3) -> bool {
    (self.x - other.x).abs() < Vec3::EPSILON
    && (self.y - other.y).abs() < Vec3::EPSILON
    && (self.z - other.z).abs() < Vec3::EPSILON
  }
}
