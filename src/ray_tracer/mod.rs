pub mod vec3;
pub use vec3::*;
pub mod objects;
pub use objects::*;
pub mod scene;
pub use scene::*;
pub mod mat44;
pub use mat44::*;
pub mod camera;
pub use camera::*;
pub mod renderer;
pub use renderer::*;

// are only used internally
mod ray;
use ray::*;
mod solver;
use solver::*;
mod viewport;
use viewport::*;

// I want to use this across the project without importing it
pub use crate::Time;
