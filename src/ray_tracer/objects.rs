use serde::{Deserialize, Serialize};
use crate::ray_tracer::{
  Vec3,
  Ray,
  solve_quadratic,
  bytes_concat_n,
};

use super::utils::tuple_bytes;

/// These parameters influence how light interacts with the object.
#[derive(Deserialize, Serialize, Clone)]
pub struct Material {
  /// The albedo colour of the object.
  ///
  /// In the order red, green, blue.
  /// 
  /// In the range 0..1.
  pub colour: (f32, f32, f32),
  /// The specularity of the object.
  /// 
  /// Values that work are from about 1..1000,
  /// with 1000 being a very shiny object.
  pub specular: f32,
  /// How much of the object's colour is a reflection of the environment.
  /// 
  /// In the range 0..1.
  pub metallic: f32,
}

impl Material {
  /// Get the byte representation of the object.
  pub fn as_bytes(&self) -> [u8; 20] {
    bytes_concat_n(&[
      &tuple_bytes::<12>(self.colour),
      &self.specular.to_le_bytes(),
      &self.metallic.to_le_bytes(),
    ])
  }
}

/// Stores the geometry of an object.
/// 
/// Each type has it's own parameters.
/// 
/// Different types are:
/// - Sphere
/// - Plane
#[derive(Deserialize, Serialize, Clone)]
pub enum Geometry {
  /// A sphere.
  Sphere {
    /// The center of the sphere.
    center: Vec3,
    /// The radius of the sphere.
    radius: f32,
  },
  /// A plane
  Plane {
    /// The center of the plane.
    center: Vec3,
    /// The normal the plane faces towards.
    normal: Vec3,
    /// The length of each side of the plane.
    size: f32,
  },
}

impl Geometry {
  /// Get the byte representation of the object.
  pub fn as_bytes(&self) -> [u8; 56] {
    match self {
      Geometry::Sphere { center, radius } => bytes_concat_n(&[
        &0u32.to_le_bytes(),
        &[0u8; 12],
        &center.as_bytes::<16>(),
        &[0u8; 12],
        &radius.to_le_bytes(),
      ]),
      Geometry::Plane { center, normal, size } => bytes_concat_n(&[
        &1u32.to_le_bytes(),
        &[0u8; 12],
        &center.as_bytes::<16>(),
        &normal.as_bytes::<12>(),
        &size.to_le_bytes(),
      ]),
    }
  }

  /// Get the closest intersection of a ray with this object.
  /// 
  /// Returns ( distance, hit point ) if hit, None otherwise.
  pub fn intersect (&self, ray: &Ray) -> Option<(f32, Vec3)> {
    match self {
      Geometry::Sphere { center, radius } => {
        // working out in whiteboard
        let new_origin = ray.origin - *center;

        let a = 1.;
        let b = 2. * ray.direction.dot(new_origin);
        let c = new_origin.dot(new_origin) - radius.powi(2);

        let solution = solve_quadratic(a, b, c);

        match solution {
          Some(solution) => {
            if solution.0 < solution.1 {
              Some((solution.0, ray.origin + (ray.direction * solution.0)))
            } else {
              Some((solution.1, ray.origin + (ray.direction * solution.1)))
            }
          }
          None => None
        }
      },
      Geometry::Plane { center, normal, size } => {
        // working out in whiteboard
        let denominator = ray.direction.dot(*normal);

        if denominator.abs() < 1e-6 {
          return None;
        }

        let numerator = (*center - ray.origin).dot(*normal);
        let t = numerator / denominator;

        let hit_point = ray.origin + (ray.direction * t);

        if (hit_point.x - center.x).abs() > *size {
          return None
        }
        if (hit_point.y - center.y).abs() > *size {
          return None
        }
        if (hit_point.z - center.z).abs() > *size {
          return None
        }

        Some((t, hit_point))
      },
    }
  }

  /// Get the normal of the surface at a point.
  pub fn normal_at_point (&self, point: Vec3) -> Vec3 {
    match self {
      Geometry::Sphere { center, radius: _ } => {
        // simple circle stuff
        (point - *center).normalize()
      },
      Geometry::Plane { center: _, normal, size: _ } => {
        // normal is the same everywhere
        *normal
      },
    }
  }

  /// Gets the position of the object to show in the editor.
  pub fn position (&self) -> &Vec3 {
    match self {
      Geometry::Sphere { center, radius: _ } => center,
      Geometry::Plane { center, normal: _, size: _ } => center,
    }
  }

  /// Gets the position of the object to show in the editor.
  pub fn position_as_mut (&mut self) -> &mut Vec3 {
    match self {
      Geometry::Sphere { center, radius: _ } => center,
      Geometry::Plane { center, normal: _, size: _ } => center,
    }
  }
}

/// Stores all the information about an object.
#[derive(Deserialize, Serialize, Clone)]
pub struct Object {
  /// The name of the object.
  /// 
  /// This doesn't have to be unique, it's just for the editor.
  pub name: String,
  /// The material of the object.
  pub material: Material,
  /// The geometry of the object.
  pub geometry: Geometry,
}

impl Object {
  /// Get the byte representation of the object.
  pub fn as_bytes(&self) -> [u8; 96] {
    bytes_concat_n(&[
      &self.material.as_bytes(),
      &[0u8; 12],
      &self.geometry.as_bytes(),
      &[0u8; 8],
    ])
  }
}

/// Stores the information about a light.
/// 
/// The different types are:
/// - Direction
/// - Point
#[derive(Deserialize, Serialize, Clone)]
pub enum Light {
  Direction { intensity: (f32, f32, f32), direction: Vec3},
  Point { intensity: (f32, f32, f32), position: Vec3},
}

impl Light {
  /// Get the byte representation of the object.
  pub fn as_bytes(&self) -> [u8; 48] {
    match self {
      Light::Direction { intensity, direction } => bytes_concat_n(&[
        &0u32.to_le_bytes(),
        &[0u8; 12],
        &tuple_bytes::<16>(*intensity),
        &direction.as_bytes::<12>(),
      ]),
      Light::Point { intensity, position } => bytes_concat_n(&[
        &1u32.to_le_bytes(),
        &[0u8; 12],
        &tuple_bytes::<16>(*intensity),
        &position.as_bytes::<12>(),
      ]),
    }
  }

  /// Get the intensity of the light.
  /// 
  /// For some types of lights (e.g. spot) this will depend on the given point.
  pub fn intensity(&self, _point: Vec3) -> (f32, f32, f32) {
    match self {
      Light::Direction { intensity, direction: _ } => *intensity,
      Light::Point { intensity, position: _ } => *intensity,
    }
  }

  /// Return a vector from the given point to the light.
  /// 
  /// For most lights this will depend on the given point,
  /// with the exception of direction which is the same at every point in a scene.
  pub fn point_to_light(&self, point: Vec3) -> Vec3 {
    match self {
      Light::Direction { intensity: _, direction } => -*direction,
      Light::Point { intensity: _, position } => *position - point,
    }
  }
}
