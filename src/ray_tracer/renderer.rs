use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::ray_tracer::*;

/// The thing that does all of the rendering.
#[derive(Clone, Deserialize, Serialize)]
pub struct Renderer {
  pub width: u32,
  pub height: u32,
  pub scene: Scene,
}

impl Renderer {
  /// Creates a new Renderer with a scene full of random spheres.
  pub fn new(width: u32, height: u32) -> Renderer {
    Renderer {
      width,
      height,
      scene: Scene::random_sphere_default_config(),
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

  /// Get the colour of the object the ray hits.
  /// 
  /// This takes into account everything, including reflections, shadows, etc.
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

  /// Renders an image to the given `ColorImage`.
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

/// Starts a thread that will continuously render an image to the given `ColorImage`.
/// 
/// Will also time how long it takes and add it to `frame_times`.
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
