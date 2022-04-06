use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use rand_distr::Distribution;
use std::sync::{Arc, Mutex};

use crate::ray_tracer::*;

#[derive(Clone, Deserialize, Serialize)]
pub struct Renderer {
  pub width: u32,
  pub height: u32,
  pub scene: Scene,
}

impl Renderer {
  pub fn new(width: u32, height: u32) -> Renderer {
    let min_radius: f64 = 3.;
    let max_radius: f64 = 8.;

    let (placement_radius, random_sphere_count) = if num_cpus::get() > 2 {
      (50., 100)
    } else {
      (20., 5)
    };

    let mut objects: Vec<Object> = vec![];

    for i in 0i32..random_sphere_count {
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
  
    Renderer {
      width,
      height,
      scene: Scene {
        camera: Camera {
          position: Vec3 { x: 5., y: 5., z: 5. },
          rotation: Vec3 { x: 0.7, y: -std::f64::consts::PI / 4., z: 0. },
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
      },
    }
  }

  /// Get the local colour, accounting for lights and shadows, nothing else.
  /// 
  /// Reflections etc. are not accounted for.
  fn calculate_local_colour(
    &self,
    point: Vec3,
    normal: Vec3,
    material: &Material,
  ) -> (f64, f64, f64) {
    // the brightness starts at the ambient light level
    let mut brightness = (
      self.scene.ambient_light.0,
      self.scene.ambient_light.1,
      self.scene.ambient_light.2,
    );

    for light in self.scene.lights.iter() {
      let point_to_light = light.point_to_light(point);

      let shadow_ray = &Ray {
        origin: point,
        direction: point_to_light.normalize(),
      };

      // ignore this light if object is in shadow
      match shadow_ray.closest_intersection(&self.scene) {
        Some((_object, _point)) => continue,
        None => (),
      }

      // get the intensity of the light,
      // for some lights this will depend on the point
      let base_intensity = light.intensity(point);

      // calculate the intensity of the light depending on the angle
      let angle_intensity = (normal.dot(point_to_light)
        / (normal.length() * point_to_light.length()))
        .clamp(0., 1.);

      // calculate the specular intensity
      let reflection_vector = Ray::get_reflection_vec(shadow_ray.direction, normal);
      let camera_vector = self.scene.camera.position - point;
      let specular = (reflection_vector.dot(camera_vector)
        / (reflection_vector.length() * camera_vector.length())).clamp(0., 1.).powf(material.specular);

      // add these all together
      brightness.0 += base_intensity.0 * (angle_intensity + specular);
      brightness.1 += base_intensity.1 * (angle_intensity + specular);
      brightness.2 += base_intensity.2 * (angle_intensity + specular);
    }

    // apply the final brightness to the colour
    (
      brightness.0 * material.colour.0,
      brightness.1 * material.colour.1,
      brightness.2 * material.colour.2,
    )
  }

  fn trace_ray(
    &self,
    ray: &Ray,
    depth: u32,
  ) -> (f64, f64, f64) {
    match ray.closest_intersection(&self.scene) {
      Some((object, hit_point)) => {
        // get the normal at the point of intersection
        let normal = object.geometry.normal_at_point(hit_point);

        // get the local colour of the object
        let local_colour = self.calculate_local_colour(hit_point, normal, &object.material);

        // if the object is not metallic, or the reflection limit is reached, return the local colour
        if object.material.metallic <= 0. || depth >= self.scene.reflection_limit {
          return local_colour;
        }

        // calculate the reflection ray
        let reflection_ray = Ray {
          origin: hit_point,
          direction: Ray::get_reflection_vec(-ray.direction, normal),
        };
        // trace the reflection ray
        let reflected_colour = self.trace_ray(&reflection_ray, depth + 1);

        // interpolate between the local colour and the reflected colour
        (
          local_colour.0 * (1. - object.material.metallic) + reflected_colour.0 * object.material.metallic,
          local_colour.1 * (1. - object.material.metallic) + reflected_colour.1 * object.material.metallic,
          local_colour.2 * (1. - object.material.metallic) + reflected_colour.2 * object.material.metallic,
        )
      },
      None => self.scene.background_colour,
    }
  }

  pub fn render(&self, image: &mut eframe::epaint::ColorImage) {
    // If the resolution of the image is incorrect, resize it
    if image.width() != self.width as usize || image.height() != self.height as usize {
      image.size = [self.width as usize, self.height as usize];
      image.pixels = vec![eframe::epaint::Color32::BLACK; (self.width * self.height) as usize];
    }

    // calculate the viewport dimensions
    let viewport = Viewport::new(&self.scene.camera, self.height as f64 / self.width as f64);

    image.pixels.par_iter_mut().enumerate().for_each(|(index, colour)| {
      // get the x and y coordinates of the pixel
      let y = (index as u32) / self.width;
      let x = index as u32 % self.width;

      // create the ray
      let ray = viewport.create_ray(
        (x as f64 + 0.5) / self.width as f64,
        (y as f64 + 0.5) / self.height as f64,
      );

      // calculate the colours of this pixel from 0..1
      let pixel = self.trace_ray(&ray, 0);

      // convert from 0..1 to a Color32
      *colour = eframe::epaint::Color32::from_rgb(
        (pixel.0 * 255.) as u8,
        (pixel.1 * 255.) as u8,
        (pixel.2 * 255.) as u8,
      );
    });
  }
}

pub fn start_render_thread(
  renderer: Arc<Mutex<Renderer>>,
  image: Arc<Mutex<eframe::epaint::ColorImage>>,
  frame_times: Arc<Mutex<eframe::egui::util::History::<f32>>>,
) {
  std::thread::spawn(move || loop {
    let start: f64 = Time::now_millis();

    // can unwrap here because if mutex is poisoned, it will panic anyway
    let renderer = renderer.lock().unwrap().clone();

    // I don't want to lock the image mutex while rendering,
    // so I clone it and draw it to that
    let mut new_image = image.lock().unwrap().clone();

    // will render to the new image
    renderer.render(&mut new_image);

    // copy the data from the new image to the global image
    let image_global = &mut image.lock().unwrap();
    image_global.size = new_image.size;
    image_global.pixels = new_image.pixels;

    // add to the frame history
    let end: f64 = Time::now_millis();
    let frame_time = end - start;
    frame_times.lock().unwrap().add(end, frame_time as f32);
  });
}
