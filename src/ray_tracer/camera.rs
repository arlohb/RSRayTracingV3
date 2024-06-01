use super::{Axis, Mat44, Vec3};

/// Stores information about the camera in a scene.
#[derive(Clone, PartialEq)]
pub struct Camera {
    /// The position of the camera.
    pub position: Vec3,
    /// The rotation of the camera.
    ///
    /// This is a euler rotation in radians.
    pub rotation: Vec3,
    /// The fov of the camera in degrees.
    pub fov: f32,
}

impl Camera {
    /// Clamps / wraps the rotation to within these limits.
    /// - x should be clamped between -pi/2 and pi/2
    /// - y should be wrapped around to between -pi and pi
    /// - z should be clamped between -pi and pi
    ///
    /// This should be called after any manipulation of the rotation.
    pub fn clamp_rotation(&mut self) {
        // x should be clamped between -pi/2 and pi/2
        self.rotation.x = self.rotation.x.clamp(
            (-0.5f32).mul_add(std::f32::consts::PI, 0.01),
            0.5f32.mul_add(std::f32::consts::PI, -0.01),
        );

        // y should be wrapped around to between -pi and pi
        if self.rotation.y < std::f32::consts::PI {
            self.rotation.y += 2. * std::f32::consts::PI;
        }
        if self.rotation.y > std::f32::consts::PI {
            self.rotation.y -= 2. * std::f32::consts::PI;
        }

        // z should be clamped between -pi and pi
        self.rotation.z = self
            .rotation
            .z
            .clamp(-std::f32::consts::PI, std::f32::consts::PI);
    }

    /// Calculates the forward, right, up vectors from the camera.
    ///
    /// This is done together as each one depends on the one before it, so this saves calculations.
    ///
    /// The 'fru' stands for forward, right, up as every time its used I need a reminder what order they are in.
    #[must_use]
    pub fn get_vectors_fru(&self) -> (Vec3, Vec3, Vec3) {
        let forward = Vec3 {
            x: 0.,
            y: 0.,
            z: 1.,
        }
        .transform_point(Mat44::create_rotation(Axis::X, -self.rotation.x))
        .transform_point(Mat44::create_rotation(Axis::Y, -self.rotation.y));

        let temp = Vec3 {
            x: 0.,
            y: 1.,
            z: 0.,
        }
        .transform_point(Mat44::create_rotation(Axis::Z, -self.rotation.z));
        let right = (temp * forward).normalize();

        let up = (forward * right).normalize();

        (forward, right, up)
    }
}
