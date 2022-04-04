use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use rand_distr::Distribution;
use std::sync::{Mutex, Arc};

use crate::ray_tracer::*;

pub struct ImagePlane {
  pub left: Vec3,
  pub right: Vec3,
  pub bottom: Vec3,
  pub top: Vec3,
}

impl ImagePlane {
  pub fn center(&self) -> Vec3 {
    (self.left + self.right) / 2.
  }
}

pub struct Hit<'a> {
  pub distance: f64,
  pub point: Vec3,
  pub object: &'a Object,
}

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

  fn get_image_plane(&self, aspect_ratio: f64) -> ImagePlane {
    // working for this is in whiteboard
    let fov_rad = self.scene.camera.fov * (std::f64::consts::PI / 180.);
    let width = 2. * f64::tan(fov_rad / 2.);
    let half_width = width / 2.;

    let height = width * aspect_ratio;
    let half_height = height / 2.;

    let (forward, right, up) = self.scene.camera.get_vectors_fru();

    // the image plane is 1 unit away from the camera
    // this is - not + because the camera point in the -forward direction
    let center = self.scene.camera.position - forward;

    ImagePlane {
      left: center - (right * half_width),
      right: center + (right * half_width),
      bottom: center - (up * half_height),
      top: center + (up * half_height),
    }
  }

  fn reflect_ray(ray: Vec3, surface_normal: Vec3) -> Vec3 {
    (surface_normal * surface_normal.dot(ray)) * 2. - ray
  }

  fn calculate_light(
    &self,
    point: Vec3,
    normal: Vec3,
    material: &Material,
  ) -> (f64, f64, f64) {
    let mut result = (
      self.scene.ambient_light.0,
      self.scene.ambient_light.1,
      self.scene.ambient_light.2,
    );

    for light in self.scene.lights.iter() {
      let point_to_light = light.point_to_light(point);

      // ignore this light if object is in shadow
      match self.ray_hit(&Ray {
        origin: point,
        direction: point_to_light.normalize(),
      }) {
        Some((_object, _point)) => continue,
        None => (),
      }

      let intensity = light.intensity(point);

      let strength = (normal.dot(point_to_light)
        / (normal.length() * point_to_light.length())).clamp(0., 1.);
      
      let reflection_vector = Renderer::reflect_ray(point_to_light.normalize(), normal);
      let camera_vector = self.scene.camera.position - point;

      let specular = (reflection_vector.dot(camera_vector)
        / (reflection_vector.length() * camera_vector.length())).clamp(0., 1.).powf(material.specular);

      result.0 += intensity.0 * (strength + specular);
      result.1 += intensity.1 * (strength + specular);
      result.2 += intensity.2 * (strength + specular);
    }

    result
  }

  fn ray_hit(
    &self,
    ray: &Ray,
  ) -> Option<(&Object, Vec3)> {
    let mut hit: Option<Hit> = None;

    for object in &self.scene.objects {
      match object.geometry.intersect(ray) {
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

    hit.map(|h| (h.object, h.point))
  }

  fn trace_ray(
    &self,
    ray: &Ray,
    depth: u32,
  ) -> (f64, f64, f64) {
    match self.ray_hit(ray) {
      Some((object, hit_point)) => {
        let normal = object.geometry.normal_at_point(hit_point);

        let brightness = self.calculate_light(hit_point, normal, &object.material);
        let local_colour = (
            brightness.0 * object.material.colour.0,
            brightness.1 * object.material.colour.1,
            brightness.2 * object.material.colour.2,
        );

        if object.material.metallic <= 0. || depth >= self.scene.reflection_limit {
          return local_colour;
        }

        let reflection_ray = Ray {
          origin: hit_point,
          direction: Renderer::reflect_ray(-ray.direction, normal),
        };
        let reflected_colour = self.trace_ray(&reflection_ray, depth + 1);

        (
          local_colour.0 * (1. - object.material.metallic) + reflected_colour.0 * object.material.metallic,
          local_colour.1 * (1. - object.material.metallic) + reflected_colour.1 * object.material.metallic,
          local_colour.2 * (1. - object.material.metallic) + reflected_colour.2 * object.material.metallic,
        )
      },
      None => self.scene.background_colour,
    }
  }

  fn render_pixel(
    &self,
    top_left: Vec3,
    x: u32,
    y: u32,
    width_world_space: f64,
    height_world_space: f64,
  ) -> (f64, f64, f64) {
    let x_screen_space = (x as f64 + 0.5) / self.width as f64;
    let y_screen_space = (y as f64 + 0.5) / self.height as f64;

    let (_, right, up) = self.scene.camera.get_vectors_fru();

    let x_offset = right * (x_screen_space * width_world_space);
    // mul -1 because it's offset down
    let y_offset = -up * (y_screen_space * height_world_space);

    let pixel_world_space = top_left + x_offset + y_offset;

    let direction = (pixel_world_space - self.scene.camera.position).normalize();

    let ray = Ray {
      origin: self.scene.camera.position,
      direction
    };

    self.trace_ray(&ray, 0)
  }

  pub fn rs_render(&self, image: &mut eframe::epaint::ColorImage) {
    if image.width() != self.width as usize || image.height() != self.height as usize {
      *image = eframe::epaint::ColorImage::new([self.width as usize, self.height as usize], eframe::epaint::Color32::BLACK);
    }

    let image_plane = self.get_image_plane(self.height as f64 / self.width as f64);

    // working for this in whiteboard
    let top_left_point = image_plane.left + image_plane.top - image_plane.center();

    let width_world_space = (image_plane.right - image_plane.left).length();
    let height_world_space = (image_plane.top - image_plane.bottom).length();

    image.pixels.par_iter_mut().enumerate().for_each(|(index, colour)| {
      let y = (index as u32) / (self.width as u32);
      let x = index as u32 % self.width;

      let pixel = self.render_pixel(top_left_point, x, y, width_world_space, height_world_space);

      *colour = eframe::epaint::Color32::from_rgb(
        (pixel.0 * 255.) as u8,
        (pixel.1 * 255.) as u8,
        (pixel.2 * 255.) as u8,
      );
    });
  }
}

pub fn render_image (
  renderer: Arc<Mutex<Renderer>>,
  image: Arc<Mutex<eframe::epaint::ColorImage>>,
  frame_times: Arc<Mutex<eframe::egui::util::History<f32>>>,
) {
  let start: f64 = Time::now_millis();

  let renderer = renderer.lock().unwrap().clone();

  let mut new_image = image.lock().unwrap().clone();

  renderer.rs_render(&mut new_image);

  let image_global = &mut image.lock().unwrap();
  image_global.size = new_image.size;
  image_global.pixels = new_image.pixels;

  let end: f64 = Time::now_millis();
  let frame_time = end - start;
  frame_times.lock().unwrap().add(end, frame_time as f32);
}
