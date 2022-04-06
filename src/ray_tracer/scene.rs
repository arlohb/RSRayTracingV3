use rand_distr::Distribution;
use serde::{Deserialize, Serialize};
use crate::ray_tracer::*;

/// Stores all the information about a scene
#[derive(Deserialize, Serialize, Clone)]
pub struct Scene {
  pub camera: Camera,
  pub objects: Vec<Object>,
  pub lights: Vec<Light>,
  pub background_colour: (f64, f64, f64),
  pub ambient_light: (f64, f64, f64),
  pub reflection_limit: u32,
  pub do_objects_spin: bool,
}

impl Scene {
  /// Randomly fills a scene with spheres using default parameters.
  /// 
  /// This will be more / less intensive depending on how many CPU cores are available.
  pub fn random_sphere_default_config() -> Scene {
    Scene::random_sphere(
      3.,
      8.,
      if num_cpus::get() > 2 { 50. } else { 20. },
      if num_cpus::get() > 2 { 100 } else { 5 },
    )
  }

  /// Randomly fills a scene with spheres using the given parameters.
  pub fn random_sphere(
    min_radius: f64,
    max_radius: f64,
    placement_radius: f64,
    sphere_count: u32,
  ) -> Scene {
    let mut objects: Vec<Object> = vec![];

    for i in 0u32..sphere_count {
      // if it failed 100 times, then there's probably no space left
      for _ in 0..100 {
        let radius: f64 = rand::random::<f64>() * (max_radius - min_radius) + min_radius;
        let [x, y]: [f64; 2] = rand_distr::UnitDisc.sample(&mut rand::thread_rng());
        let x = x * placement_radius;
        let y = y * placement_radius;
        let position = Vec3 { x, y: radius, z: y };
        
        // reject spheres that are intersecting others
        if objects.iter().any(|object| {
          let other_radius = match object.geometry {
            Geometry::Sphere { radius, .. } => radius,
            _ => return false,
          };
          let min_dst = radius + other_radius;
          (*object.geometry.position() - position).length() < min_dst
        }) {
          continue;
        }

        objects.push(Object {
          name: i.to_string(),
          material: Material {
            colour: (rand::random(), rand::random(), rand::random()),
            // some sort of distribution would be better here
            specular: rand::random::<f64>() * 1000.,
            metallic: if rand::random::<f64>() > 0.3 { rand::random() } else { 0. },
          },
          geometry: Geometry::Sphere {
            center: position,
            radius,
          },
        });

        break;
      }
    }

    objects.push(Object {
      name: "plane".to_string(),
      geometry: Geometry::Plane {
        center: Vec3 { x: 0., y: 0., z: 0. },
        normal: Vec3 { x: 0., y: 1., z: 0. },
        size: 100000.,
      },
      material: Material {
        colour: (0.5, 0.5, 0.5),
        specular: 10.,
        metallic: 0.2,
      },
    });

    Scene {
      camera: Camera {
        position: Vec3 { x: 55., y: 65., z: 55. },
        rotation: Vec3 { x: 0.8, y: -0.8, z: 0. },
        fov: 70.,
      },
      objects,
      lights: vec![
        Light::Direction {
          intensity: (0.4, 0.4, 0.4),
          direction: Vec3 { x: -1., y: -1.5, z: -0.5 }.normalize(),
        },
        Light::Point {
          intensity: (0.4, 0.4, 0.4),
          position: Vec3 { x: 0., y: 2., z: 0., },
        },
      ],
      background_colour: (0.5, 0.8, 1.),
      ambient_light: (0.2, 0.2, 0.2),
      reflection_limit: 4,
      do_objects_spin: false,
    }
  }
}
