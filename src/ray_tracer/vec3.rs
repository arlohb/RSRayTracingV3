use std::ops;

use super::Mat44;
use crate::utils::bytes::*;

/// Represents a point, vector, or normal in 3D space.
#[derive(Debug, Copy, Clone)]
pub struct Vec3 {
  pub x: f32,
  pub y: f32,
  pub z: f32,
}

impl Vec3 {
  /// A tiny value used when checking for equality.
  const EPSILON: f32 = 0.000001;

  /// Get the byte representation of the object.
  pub fn as_bytes<const N: usize>(&self) -> [u8; N] {
    bytes_concat_n(&[
      &self.x.to_le_bytes(),
      &self.y.to_le_bytes(),
      &self.z.to_le_bytes(),
    ])
  }

  /// The length of a vector.
  pub fn length(&self) -> f32 {
    ((self.x.powi(2)) + (self.y.powi(2)) + (self.z.powi(2))).sqrt()
  }

  /// The dot product between self and v.
  pub fn dot(&self, v: Vec3) -> f32 {
    (self.x * v.x) 
      + (self.y * v.y)
      + (self.z * v.z)
  }

  /// Change this vector to have a length of 1
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

  /// Transform a point by a given matrix.
  /// 
  /// This does not do position, it may need to in the future though.
  pub fn transform_point(&self, mat: Mat44) -> Vec3 {
    Vec3 {
      x: self.x * mat[0][0] + self.y * mat[1][0] + self.z * mat[2][0],
      y: self.x * mat[0][1] + self.y * mat[1][1] + self.z * mat[2][1],
      z: self.x * mat[0][2] + self.y * mat[1][2] + self.z * mat[2][2],
    }
  }

  /// Gets the fractional part of each component.
  pub fn fract(&self) -> Vec3 {
    Vec3 {
      x: self.x.fract(),
      y: self.y.fract(),
      z: self.z.fract(),
    }
  }

  /// Gets the absolute value of each component
  pub fn abs(&self) -> Vec3 {
    Vec3 {
      x: self.x.abs(),
      y: self.y.abs(),
      z: self.z.abs(),
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

impl ops::Mul<f32> for Vec3 {
  type Output = Vec3;
  fn mul(self, rhs: f32) -> Vec3 {
    Vec3 {
      x: self.x * rhs,
      y: self.y * rhs,
      z: self.z * rhs,
    }
  }
}

impl ops::MulAssign<f32> for Vec3 {
  fn mul_assign(&mut self, rhs: f32) {
    *self = Vec3 {
      x: self.x * rhs,
      y: self.y * rhs,
      z: self.z * rhs,
    }
  }
}

impl ops::Div<f32> for Vec3 {
  type Output = Vec3;
  fn div(self, rhs: f32) -> Vec3 {
    Vec3 {
      x: self.x / rhs,
      y: self.y / rhs,
      z: self.z / rhs,
    }
  }
}

impl ops::DivAssign<f32> for Vec3 {
  fn div_assign(&mut self, rhs: f32) {
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
