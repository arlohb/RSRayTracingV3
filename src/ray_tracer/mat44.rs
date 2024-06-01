use std::ops;

/// An enum for a choice of axis, X, Y, Z.
#[derive(Clone, Copy)]
pub enum Axis {
    X,
    Y,
    Z,
}

/// A 4x4 matrix.
///
/// Right now this is only used to rotate points around an axis.
#[derive(Clone, Copy)]
pub struct Mat44 {
    data: [[f32; 4]; 4],
}

impl Mat44 {
    /// Create a new matrix with the given values.
    #[must_use]
    pub const fn new(data: [[f32; 4]; 4]) -> Self {
        Self { data }
    }

    /// Create a matrix representing a rotation around an axis.
    #[must_use]
    pub fn create_rotation(axis: Axis, radians: f32) -> Self {
        match axis {
            Axis::X => Self::new([
                [1., 0., 0., 0.],
                [0., radians.cos(), radians.sin(), 0.],
                [0., -radians.sin(), radians.cos(), 0.],
                [0., 0., 0., 1.],
            ]),
            Axis::Y => Self::new([
                [radians.cos(), 0., -radians.sin(), 0.],
                [0., 1., 0., 0.],
                [radians.sin(), 0., radians.cos(), 0.],
                [0., 0., 0., 1.],
            ]),
            Axis::Z => Self::new([
                [radians.cos(), radians.sin(), 0., 0.],
                [-radians.sin(), radians.cos(), 0., 0.],
                [0., 0., 1., 0.],
                [0., 0., 0., 1.],
            ]),
        }
    }
}

impl ops::Index<usize> for Mat44 {
    type Output = [f32; 4];
    fn index(&self, index: usize) -> &[f32; 4] {
        &self.data[index]
    }
}
