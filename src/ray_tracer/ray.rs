use crate::ray_tracer::*;

/// Represents an intersection between a ray and an object.
struct Hit<'a> {
  /// The distance along a ray at which the intersection is.
  pub distance: f64,
  /// The intersection point.
  pub point: Vec3,
  /// The object that was hit.
  pub object: &'a Object,
}

/// Represents a ray.
pub struct Ray {
  pub origin: Vec3,
  pub direction: Vec3,
}

impl Ray {
  /// Return the direction of a ray reflected off a surface with the given normal.
  pub fn get_reflection_vec(ray_dir: Vec3, surface_normal: Vec3) -> Vec3 {
    (surface_normal * surface_normal.dot(ray_dir)) * 2. - ray_dir
  }

  /// Return the object and intersection point that the ray hits first.
  pub fn closest_intersection<'a>(&self, scene: &'a Scene) -> Option<(&'a Object, Vec3)> {
    let mut hit: Option<Hit> = None;

    for object in &scene.objects {
      match object.geometry.intersect(self) {
        Some((distance, hit_point)) => {
          if distance < 1e-6 { continue }
          match &hit {
            Some(h) => {
              if distance < h.distance {
                hit = Some(Hit {
                  distance,
                  point: hit_point,
                  object,
                });
              }
            },
            None => {
              hit = Some(Hit {
                distance,
                point: hit_point,
                object,
              });
            }
          }
        },
        None => continue
      };      
    }

    match hit {
      Some(hit) => Some((hit.object, hit.point)),
      None => None
    }
  }
}
