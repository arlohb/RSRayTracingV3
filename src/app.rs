use rand_distr::{UnitDisc, Distribution};
use eframe::{egui, epi};
use std::sync::{Mutex, Arc};
use crate::{ray_tracer::*, panels::*, Time};

pub struct App {
  renderer: Renderer,
  texture: Option<eframe::epaint::TextureHandle>,
  last_time: f64,
  g_renderer: Arc<Mutex<Options>>,
  image: Arc<Mutex<eframe::epaint::ColorImage>>,
  frame_times: Arc<Mutex<eframe::egui::util::History<f32>>>,
}

impl App {
  pub fn new(
    width: u32,
    height: u32,
    options: Arc<Mutex<Options>>,
    image: Arc<Mutex<eframe::epaint::ColorImage>>,
    frame_times: Arc<Mutex<eframe::egui::util::History<f32>>>,
  ) -> Self {
    let min_radius: f64 = 3.;
    let max_radius: f64 = 8.;
    let placement_radius = 50.;
    let random_sphere_count = 100;

    let mut objects: Vec<Object> = vec![];

    for i in 0..random_sphere_count {
      // if it failed 100 times, then there's probably no space left
      for _ in 0..100 {
        let radius: f64 = rand::random::<f64>() * (max_radius - min_radius) + min_radius;
        let [x, y]: [f64; 2] = UnitDisc.sample(&mut rand::thread_rng());
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
  
    Self {
      renderer: Renderer {
        camera: Vec3 { x: 5., y: 5., z: 5. },
        rotation: Vec3 { x: 0.7, y: -std::f64::consts::PI / 4., z: 0. },
        fov: 70.,
        width,
        height,
        scene: Scene {
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
      },
      texture: None,
      last_time: Time::now(),
      g_renderer: options,
      image,
      frame_times,
    }
  }
}

impl epi::App for App {
  fn name(&self) -> &str {
    "RSRayTracingV2"
  }

  /// Called once before the first frame.
  fn setup(
    &mut self,
    ctx: &egui::Context,
    _frame: &epi::Frame,
    _storage: Option<&dyn epi::Storage>,
  ) {
    ctx.set_style({
      let mut style: egui::Style = (*ctx.style()).clone();
      style.visuals = egui::Visuals::dark();
      style
    });
  }

  /// Called each time the UI needs repainting, which may be many times per second.
  /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
  fn update(&mut self, ctx: &egui::Context, _: &epi::Frame) {    
    let screen_rect = ctx.input().screen_rect;
    let is_portrait = screen_rect.height() > screen_rect.width();
    
    let mut has_size_changed = false;

    let fps = match self.frame_times.try_lock() {
      Ok(times) => {
        1000. / times.average().unwrap_or(10.)
      },
      Err(_) => 1.,
    };

    let now = Time::now();
    // delta_time is in seconds
    let delta_time = (now - self.last_time) / 1000.;
    self.last_time = now;

    {
      let renderer = &mut self.renderer;

      crate::movement::move_and_rotate(
        &ctx.input(),
        renderer,
        delta_time * 1.5,
        delta_time * 20.,
        6.,
        0.4,
      );

      if renderer.scene.do_objects_spin {
        renderer.scene.objects.iter_mut().for_each(|object| {
          if !matches!(object.geometry, Geometry::Sphere { .. }) {
            return;
          }

          let position = object.geometry.position_as_mut();
          let length = position.length();

          let theta: f64 = 0.5 * std::f64::consts::PI * delta_time;

          *position = position.transform_point(Mat44::create_rotation(Axis::Y, theta));

          // fix rounding errors?
          *position *= length / position.length();
        });
      }
    }

    if is_portrait {
      egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
        egui::SidePanel::left("object_panel")
          .show_inside(ui, |ui| object_panel(ui, &mut self.renderer.scene));
        egui::SidePanel::right("settings_panel")
          .show_inside(ui, |ui| settings_panel(ui, fps, &mut self.renderer, &mut has_size_changed));
      });
    } else {
      egui::SidePanel::right("settings_panel")
        .show(ctx, |ui| settings_panel(ui, fps, &mut self.renderer, &mut has_size_changed));
      egui::SidePanel::right("object_panel")
        .show(ctx, |ui| object_panel(ui, &mut self.renderer.scene));
    }

    egui::CentralPanel::default().show(ctx, |ui| {
      ui.set_max_width(f32::INFINITY);
      ui.set_max_height(f32::INFINITY);
      let texture = &mut self.texture;
      match texture {
        Some(texture) => {
          egui::Resize::default()
            .default_size((self.renderer.width as f32, self.renderer.height as f32))
            .show(ui, |ui| {
              if !has_size_changed {
                let renderer = &mut self.renderer;
                renderer.width = ui.available_width() as u32;
                renderer.height = ui.available_height() as u32;
              }

              if let Ok(image) = self.image.try_lock() {
                texture.set(eframe::epaint::ImageData::Color(image.clone()));
              }

              ui.add(egui::Image::new(texture.id(), texture.size_vec2()));
            });
        },
        None => {
          let image = eframe::epaint::ColorImage::new(
            [self.renderer.width as usize, self.renderer.height as usize],
            eframe::epaint::Color32::BLACK
          );
          self.texture = Some(ctx.load_texture("canvas", image));
        },
      }
    });

    if let Ok(mut options) = self.g_renderer.try_lock() {
      options.camera = self.renderer.camera;
      options.rotation = self.renderer.rotation;
      options.fov = self.renderer.fov;
      options.width = self.renderer.width;
      options.height = self.renderer.height;
      options.scene = self.renderer.scene.clone()
    }

    ctx.request_repaint();
  }
}
