use crate::ray_tracer::*;

pub struct Hit<'a> {
  pub distance: f64,
  pub point: Vec3,
  pub object: &'a Object,
}

pub struct Ray {
  pub origin: Vec3,
  pub direction: Vec3,
}

impl Ray {
  pub fn get_reflection_vec(ray_dir: Vec3, surface_normal: Vec3) -> Vec3 {
    (surface_normal * surface_normal.dot(ray_dir)) * 2. - ray_dir
  }

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
