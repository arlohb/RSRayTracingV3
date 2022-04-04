pub mod vec3;
pub use vec3::*;
pub mod ray;
pub use ray::*;
pub mod objects;
pub use objects::*;
pub mod solver;
pub use solver::*;
pub mod scene;
pub use scene::*;
pub mod mat44;
pub use mat44::*;
pub mod camera;
pub use camera::*;
pub mod renderer;
pub use renderer::*;

// I want to use this across the project without importing it
pub use crate::Time;
