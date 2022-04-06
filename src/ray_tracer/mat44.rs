use std::ops;

/// An enum for a choice of axis, X, Y, Z.
pub enum Axis {
  X,
  Y,
  Z,
}

/// A 4x4 matrix.
/// 
/// Right now this is only used to rotate points around an axis.
pub struct Mat44 {
  data: [[f64; 4]; 4],
}

impl Mat44 {
  /// Create a new matrix with the given values.
  pub fn new(data: [[f64; 4]; 4]) -> Mat44 {
    Mat44 {
      data,
    }
  }

  /// Create a matrix representing a rotation around an axis.
  pub fn create_rotation(axis: Axis, radians: f64) -> Mat44 {
    match axis {
      Axis::X => Mat44::new([
        [1., 0., 0., 0.],
        [0., radians.cos(), radians.sin(), 0.],
        [0., -radians.sin(), radians.cos(), 0.],
        [0., 0., 0., 1.],
      ]),
      Axis::Y => Mat44::new([
        [radians.cos(), 0., -radians.sin(), 0.],
        [0., 1., 0., 0.],
        [radians.sin(), 0., radians.cos(), 0.],
        [0., 0., 0., 1.],
      ]),
      Axis::Z => Mat44::new([
        [radians.cos(), radians.sin(), 0., 0.],
        [-radians.sin(), radians.cos(), 0., 0.],
        [0., 0., 1., 0.],
        [0., 0., 0., 1.],
      ]),
    }
  }
}

impl ops::Index<usize> for Mat44 {
  type Output = [f64; 4];
  fn index(&self, index: usize) -> &[f64; 4] {
    &self.data[index]
  }
}
