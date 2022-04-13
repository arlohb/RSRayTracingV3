use rand_distr::Distribution;
use serde::{Deserialize, Serialize};
use crate::ray_tracer::*;

/// Stores all the information about a scene
#[derive(Deserialize, Serialize, Clone)]
pub struct Scene {
  pub camera: Camera,
  pub objects: Vec<Object>,
  pub lights: Vec<Light>,
  pub background_colour: (f32, f32, f32),
  pub ambient_light: (f32, f32, f32),
  pub reflection_limit: u32,
  pub do_objects_spin: bool,
}

impl Scene {
  pub const BUFFER_SIZE: (u32, u32, u32) = (
    96 * 80, // allows for 80 objects
    48 * 2, // allows for 2 lights
    112,
  );

  /// Returns a simple scene with a single sphere and light
  pub fn simple() -> Scene {
    Scene {
      objects: vec![
        Object {
          name: "Sphere".to_string(),
          geometry: Geometry::Sphere {
            center: Vec3 { x: 0., y: 0., z: 0. },
            radius: 1.,
          },
          material: Material {
            colour: (1., 0., 0.),
            specular: 100.,
            metallic: 1.
          },
        },
      ],
      lights: vec![
        Light::Point {
          intensity: (1., 1., 1.),
          position: Vec3 { x: 0., y: 2., z: 0. },
        },
      ],
      camera: Camera {
        position: Vec3 { x: 0., y: 0., z: -5. },
        rotation: Vec3 { x: 0., y: 0., z: 0. },
        fov: 70.,
      },
      background_colour: (0.5, 0.8, 1.),
      ambient_light: (0.2, 0.2, 0.2),
      reflection_limit: 4,
      do_objects_spin: false,
    }
  }

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
    min_radius: f32,
    max_radius: f32,
    placement_radius: f32,
    sphere_count: u32,
  ) -> Scene {
    let mut objects: Vec<Object> = vec![];

    for i in 0u32..sphere_count {
      // if it failed 100 times, then there's probably no space left
      for _ in 0..100 {
        let radius: f32 = rand::random::<f32>() * (max_radius - min_radius) + min_radius;
        let [x, y]: [f32; 2] = rand_distr::UnitDisc.sample(&mut rand::thread_rng());
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
            specular: rand::random::<f32>() * 1000.,
            metallic: if rand::random::<f32>() > 0.3 { rand::random() } else { 0. },
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

  pub fn as_bytes(&self, width: u32, height: u32) -> (
    [u8; Scene::BUFFER_SIZE.0 as usize],
    [u8; Scene::BUFFER_SIZE.1 as usize],
    [u8; Scene::BUFFER_SIZE.2 as usize],
  ) {
    let vectors = self.camera.get_vectors_fru();

    (
      bytes_concat_fixed_in_n(self.objects
        .iter()
        .map(|object| object.as_bytes())
        .collect::<Vec<_>>()
        .as_slice()
      ),
      bytes_concat_fixed_in_n(self.lights
        .iter()
        .map(|light| light.as_bytes())
        .collect::<Vec<_>>()
        .as_slice()
      ),
      bytes_concat_n(&[
        &self.camera.position.as_bytes::<16>(),
        &vectors.0.as_bytes::<16>(),
        &vectors.1.as_bytes::<16>(),
        &vectors.2.as_bytes::<16>(),
        &tuple_bytes::<16>(self.background_colour),
        &tuple_bytes::<12>(self.ambient_light),
        &self.camera.fov.to_le_bytes(),
        &self.reflection_limit.to_le_bytes(),
        &width.to_le_bytes(),
        &height.to_le_bytes(),
        &[0u8; 4]
      ]),
    )
  }
}

impl PartialEq for Scene {
  fn eq(&self, other: &Scene) -> bool {
    self.camera == other.camera &&
    {
      if self.objects.len() != other.objects.len() {
        return false;
      } else {
        for (i, object) in self.objects.iter().enumerate() {
          if *object != other.objects[i] {
            return false;
          }
        }

        return true;
      }
    } &&
    {
      if self.lights.len() != other.lights.len() {
        return false;
      } else {
        for (i, light) in self.lights.iter().enumerate() {
          if *light != other.lights[i] {
            return false;
          }
        }

        return true;
      }
    } &&
    self.background_colour == other.background_colour &&
    self.ambient_light == other.ambient_light &&
    self.reflection_limit == other.reflection_limit &&
    self.do_objects_spin == other.do_objects_spin
  }
}
