/// An alias for `nalgebra::Vector3<f32>`
pub type Vec3 = nalgebra::Vector3<f32>;

mod objects;
pub use objects::*;
mod scene;
pub use scene::*;
mod camera;
pub use camera::*;
