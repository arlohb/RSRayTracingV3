use serde::{Deserialize, Serialize};
use crate::ray_tracer::{
  Vec3,
  Ray,
  solve_quadratic,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Material {
  pub colour: (f64, f64, f64),
  pub specular: f64,
  pub metallic: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Geometry {
  Sphere {
    center: Vec3,
    radius: f64,
  },
  Plane {
    center: Vec3,
    normal: Vec3,
    size: f64, // this is the length of each side
  },
}

impl Geometry {
  pub fn intersect (&self, ray: &Ray) -> Option<(f64, Vec3)> {
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

  pub fn position (&self) -> &Vec3 {
    match self {
      Geometry::Sphere { center, radius: _ } => center,
      Geometry::Plane { center, normal: _, size: _ } => center,
    }
  }

  pub fn position_as_mut (&mut self) -> &mut Vec3 {
    match self {
      Geometry::Sphere { center, radius: _ } => center,
      Geometry::Plane { center, normal: _, size: _ } => center,
    }
  }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Object {
  pub name: String,
  pub material: Material,
  pub geometry: Geometry,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Light {
  Direction { intensity: (f64, f64, f64), direction: Vec3},
  Point { intensity: (f64, f64, f64), position: Vec3},
}

impl Light {
  pub fn intensity(&self, _point: Vec3) -> (f64, f64, f64) {
    match self {
      Light::Direction { intensity, direction: _ } => *intensity,
      Light::Point { intensity, position: _ } => *intensity,
    }
  }

  pub fn point_to_light(&self, point: Vec3) -> Vec3 {
    match self {
      Light::Direction { intensity: _, direction } => -*direction,
      Light::Point { intensity: _, position } => *position - point,
    }
  }
}
