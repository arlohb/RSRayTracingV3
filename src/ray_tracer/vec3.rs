use std::ops;

use super::Mat44;
use crate::utils::bytes::bytes_concat_n;

// TODO: Consider just using someone else's crate

/// Represents a point, vector, or normal in 3D space.
#[derive(Debug, Copy, Clone)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    /// A tiny value used when checking for equality.
    const EPSILON: f32 = 0.000_001;

    /// Create a new `Vec3`
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Get the byte representation of the object.
    #[must_use]
    pub fn as_bytes<const N: usize>(&self) -> [u8; N] {
        bytes_concat_n(&[
            &self.x.to_le_bytes(),
            &self.y.to_le_bytes(),
            &self.z.to_le_bytes(),
        ])
    }

    /// The length of a vector.
    #[must_use]
    pub fn length(&self) -> f32 {
        // The 'performant' equivalent of this
        // ((self.x.powi(2)) + (self.y.powi(2)) + (self.z.powi(2))).sqrt()
        self.x
            .mul_add(self.x, self.y.mul_add(self.y, self.z * self.z))
            .sqrt()
    }

    /// The dot product between self and v.
    #[must_use]
    pub fn dot(&self, v: Self) -> f32 {
        // The 'performant' equivalent of this
        // (self.x * v.x) + (self.y * v.y) + (self.z * v.z)
        self.x.mul_add(v.x, self.y.mul_add(v.y, self.z * v.z))
    }

    /// Change this vector to have a length of 1
    #[must_use]
    pub fn normalize(&self) -> Self {
        let length_squared = self.dot(*self);

        if length_squared > 0. {
            let inverse_length = 1. / length_squared.sqrt();
            return Self {
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
    #[must_use]
    pub fn transform_point(&self, mat: Mat44) -> Self {
        Self {
            // The 'performant' equivalent of this
            // x: self.x * mat[0][0] + self.y * mat[1][0] + self.z * mat[2][0],
            // y: self.x * mat[0][1] + self.y * mat[1][1] + self.z * mat[2][1],
            // z: self.x * mat[0][2] + self.y * mat[1][2] + self.z * mat[2][2],
            x: self
                .x
                .mul_add(mat[0][0], self.y.mul_add(mat[1][0], self.z * mat[2][0])),
            y: self
                .x
                .mul_add(mat[0][1], self.y.mul_add(mat[1][1], self.z * mat[2][1])),
            z: self
                .x
                .mul_add(mat[0][2], self.y.mul_add(mat[1][2], self.z * mat[2][2])),
        }
    }

    /// Gets the fractional part of each component.
    #[must_use]
    pub fn fract(&self) -> Self {
        Self {
            x: self.x.fract(),
            y: self.y.fract(),
            z: self.z.fract(),
        }
    }

    /// Gets the absolute value of each component
    #[must_use]
    pub fn abs(&self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
        }
    }
}

impl ops::Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl ops::AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl ops::Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl ops::Mul<Self> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self {
            x: self.y.mul_add(rhs.z, -(self.z * rhs.y)),
            y: self.z.mul_add(rhs.x, -(self.x * rhs.z)),
            z: self.x.mul_add(rhs.y, -(self.y * rhs.x)),
        }
    }
}

impl ops::MulAssign<Self> for Vec3 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.y.mul_add(rhs.z, -(self.z * rhs.y)),
            y: self.z.mul_add(rhs.x, -(self.x * rhs.z)),
            z: self.x.mul_add(rhs.y, -(self.y * rhs.x)),
        }
    }
}

impl ops::Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl ops::MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) {
        *self = Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl ops::Div<f32> for Vec3 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl ops::DivAssign<f32> for Vec3 {
    fn div_assign(&mut self, rhs: f32) {
        *self = Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl PartialEq for Vec3 {
    fn eq(&self, other: &Self) -> bool {
        (self.x - other.x).abs() < Self::EPSILON
            && (self.y - other.y).abs() < Self::EPSILON
            && (self.z - other.z).abs() < Self::EPSILON
    }
}
