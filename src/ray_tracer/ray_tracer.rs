// #[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{ray_tracer::*, Time};

pub struct ImagePlane {
  pub left: Vec3,
  pub right: Vec3,
  pub bottom: Vec3,
  pub top: Vec3,
  pub center: Vec3,
}

pub struct Hit<'a> {
  pub distance: f64,
  pub point: Vec3,
  pub object: &'a Object,
}

pub struct RayTracer {
  pub camera: Vec3,
  pub rotation: Vec3,
  pub fov: f64,
  pub width: u32,
  pub height: u32,
  pub scene: Scene,
}

impl RayTracer {
  pub fn forward(&self) -> Vec3 {
    Vec3 { x: 0., y: 0., z: 1. }
      .transform_point(Mat44::create_rotation(Axis::X, -self.rotation.x))
      .transform_point(Mat44::create_rotation(Axis::Y, -self.rotation.y))
  }

  pub fn right(&self) -> Vec3 {
    let temp = Vec3 { x: 0., y: 1., z: 0. }
      .transform_point(Mat44::create_rotation(Axis::Z, -self.rotation.z));
    (temp * self.forward()).normalize()
  }

  pub fn up(&self) -> Vec3 {
    (self.forward() * self.right()).normalize()
  }

  fn get_image_plane(&self, aspect_ratio: f64) -> ImagePlane {
    // working for this is in whiteboard
    let fov_rad = self.fov * (std::f64::consts::PI / 180.);
    let width = 2. * f64::tan(fov_rad / 2.);
    let half_width = width / 2.;

    let height = width * aspect_ratio;
    let half_height = height / 2.;

    let right = self.right();
    let up = self.up();
    let forward = self.forward();

    // the image plane is 1 unit away from the camera
    // this is - not + because the camera point in the -forward direction
    let center = self.camera - forward;

    ImagePlane {
      left: center - (right * half_width),
      right: center + (right * half_width),
      bottom: center - (up * half_height),
      top: center + (up * half_height),
      center,
    }
  }

  fn reflect_ray(ray: Vec3, surface_normal: Vec3) -> Vec3 {
    (surface_normal * surface_normal.dot(ray)) * 2. - ray
  }

  fn calculate_light(
    &self,
    point: Vec3,
    normal: Vec3,
    camera_pos: Vec3,
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
      
      let reflection_vector = RayTracer::reflect_ray(point_to_light.normalize(), normal);
      let camera_vector = camera_pos - point;

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

    match &hit {
      Some(h) => Some((h.object, h.point)),
      None => None,
    }
  }

  fn trace_ray(
    &self,
    ray: &Ray,
    depth: u32,
  ) -> (f64, f64, f64) {
    match self.ray_hit(&ray) {
      Some((object, hit_point)) => {
        let normal = object.geometry.normal_at_point(hit_point);

        let brightness = self.calculate_light(hit_point, normal, self.camera, &object.material);
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
          direction: RayTracer::reflect_ray(-ray.direction, normal),
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
    x: u32,
    y: u32,
    top_left: Vec3,
    width_world_space: f64,
    height_world_space: f64,
    right: Vec3,
    up: Vec3,
  ) -> (f64, f64, f64) {
    let x_screen_space = (x as f64 + 0.5) / self.width as f64;
    let y_screen_space = (y as f64 + 0.5) / self.height as f64;

    let x_offset = right * (x_screen_space * width_world_space);
    // mul -1 because it's offset down
    let y_offset = (-up) * (y_screen_space * height_world_space);

    let pixel_world_space = top_left + x_offset + y_offset;

    let direction = (pixel_world_space - self.camera).normalize();

    let ray = Ray {
      origin: self.camera,
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
    let top_left_point = image_plane.left + image_plane.top - image_plane.center;

    let width_world_space = (image_plane.right - image_plane.left).length();
    let height_world_space = (image_plane.top - image_plane.bottom).length();
    let right = self.right();
    let up = self.up();

    image.pixels.par_iter_mut().enumerate().for_each(|(index, colour)| {
      let y = (index as u32) / (self.width as u32);
      let x = index as u32 % self.width;

      let pixel = self.render_pixel(x, y, top_left_point, width_world_space, height_world_space, right, up);

      *colour = eframe::epaint::Color32::from_rgb(
        (pixel.0 * 255.) as u8,
        (pixel.1 * 255.) as u8,
        (pixel.2 * 255.) as u8,
      );
    });
  }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Options {
  pub camera: Vec3,
  pub rotation: Vec3,
  pub fov: f64,
  pub width: u32,
  pub height: u32,
  pub scene: Scene,
}

impl Options {
  pub fn new(width: u32, height: u32) -> Options {
    Options {
      camera: Vec3 { x: 5., y: 5., z: 5. },
      rotation: Vec3 { x: 0.7, y: -std::f64::consts::PI / 4., z: 0. },
      fov: 70.,
      width,
      height,
      scene: Scene {
        objects: vec![
          Object {
            name: "sphere".to_string(),
            material: Material {
              colour: (
                1.0,
                0.5212054252624512,
                0.0,
              ),
              specular: 5.0,
              metallic: 1.0,
            },
            geometry: Geometry::Sphere {
              center: Vec3 {
                x: 1.5,
                y: 0.0,
                z: 0.0,
              },
              radius: 1.0,
            },
          },
          Object {
            name: "sphere".to_string(),
            material: Material {
              colour: (
                1.0,
                0.3486607074737549,
                0.0,
              ),
              specular: 800.0,
              metallic: 0.2,
            },
            geometry: Geometry::Sphere {
              center: Vec3 {
                x: 3.1,
                y: 0.0,
                z: 2.1,
              },
              radius: 1.0,
            },
          },
          Object {
            name: "sphere".to_string(),
            material: Material {
              colour: (
                0.0,
                0.6445307731628418,
                1.0,
              ),
              specular: 80.0,
              metallic: 0.,
            },
            geometry: Geometry::Sphere {
              center: Vec3 {
                x: -8.3,
                y: 0.0,
                z: 0.0,
              },
              radius: 1.0,
            },
          },
          Object {
            name: "plane".to_string(),
            material: Material {
              colour: (0.8, 0.8, 1.),
              specular: 50.,
              metallic: 0.2,
            },
            geometry: Geometry::Plane {
              center: Vec3 { x: 0., y: -1.5, z: 0. },
              normal: Vec3 { x: 0., y: 1., z: 0. },
              size: 5.,
            },
          },
        ],
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
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn render_image () {
  let start: f64 = Time::now();

  let options = crate::OPTIONS.lock().unwrap().clone();

  let ray_tracer = RayTracer {
    camera: options.camera,
    rotation: options.rotation,
    fov: options.fov,
    width: options.width,
    height: options.height,
    scene: options.scene,
  };

  let mut image = crate::IMAGE.lock().unwrap().clone();

  ray_tracer.rs_render(&mut image);

  let image_global = &mut crate::IMAGE.lock().unwrap();
  image_global.size = image.size;
  image_global.pixels = image.pixels;

  let end: f64 = Time::now();
  let frame_time = end - start;
  crate::FRAME_TIMES.lock().unwrap().add(end, frame_time as f32);
}
