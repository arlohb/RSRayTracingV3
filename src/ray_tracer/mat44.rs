use std::ops;

pub enum Axis {
  X,
  Y,
  Z,
}

pub struct Mat44 {
  data: [[f64; 4]; 4],
}

impl Mat44 {
  pub fn new(data: [[f64; 4]; 4]) -> Mat44 {
    Mat44 {
      data,
    }
  }

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
