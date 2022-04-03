use rand_distr::{UnitDisc, Distribution};
use eframe::{egui, epi};
use crate::{ray_tracer::*, panels::*, Time};

pub struct App {
  ray_tracer: RayTracer,
  texture: Option<eframe::epaint::TextureHandle>,
  last_time: f64,
}

impl App {
  pub fn new(width: u32, height: u32) -> Self {
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
      ray_tracer: RayTracer {
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

    let fps = match crate::FRAME_TIMES.try_lock() {
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
      let ray_tracer = &mut self.ray_tracer;
      
      let forward = ray_tracer.forward();
      let right = ray_tracer.right();
      let up = ray_tracer.up();

      crate::movement::move_and_rotate(
        &ctx.input(),
        &mut ray_tracer.camera,
        &mut ray_tracer.rotation,
        forward,
        right,
        up,
        delta_time * 1.5,
        delta_time * 4.,
      );

      if ray_tracer.scene.do_objects_spin {
        ray_tracer.scene.objects.iter_mut().for_each(|object| {
          let position = object.geometry.position_as_mut();
          let length = position.length();

          let theta: f64 = 0.5 * std::f64::consts::PI * delta_time;

          *position = position.transform_point(Mat44::create_rotation(Axis::Y, theta));

          // fix rounding errors?
          *position = *position * (length / position.length());
        });
      }
    }

    if is_portrait {
      egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
        egui::SidePanel::left("object_panel")
          .show_inside(ui, |ui| object_panel(ui, &mut self.ray_tracer.scene));
        egui::SidePanel::right("settings_panel")
          .show_inside(ui, |ui| settings_panel(ui, fps, &mut self.ray_tracer, &mut has_size_changed));
      });
    } else {
      egui::SidePanel::right("settings_panel")
        .show(ctx, |ui| settings_panel(ui, fps, &mut self.ray_tracer, &mut has_size_changed));
      egui::SidePanel::right("object_panel")
        .show(ctx, |ui| object_panel(ui, &mut self.ray_tracer.scene));
    }

    egui::CentralPanel::default().show(ctx, |ui| {
      ui.set_max_width(f32::INFINITY);
      ui.set_max_height(f32::INFINITY);
      let texture = &mut self.texture;
      match texture {
        Some(texture) => {
          egui::Resize::default()
            .default_size((self.ray_tracer.width as f32, self.ray_tracer.height as f32))
            .show(ui, |ui| {
              if !has_size_changed {
                let ray_tracer = &mut self.ray_tracer;
                ray_tracer.width = ui.available_width() as u32;
                ray_tracer.height = ui.available_height() as u32;
              }

              match crate::IMAGE.try_lock() {
                Ok(image) => {
                  texture.set(eframe::epaint::ImageData::Color(image.clone()));
                },
                Err(_) => (),
              };

              ui.add(egui::Image::new(texture.id(), texture.size_vec2()));
            });
        },
        None => {
          let image = eframe::epaint::ColorImage::new(
            [self.ray_tracer.width as usize, self.ray_tracer.height as usize],
            eframe::epaint::Color32::BLACK
          );
          self.texture = Some(ctx.load_texture("canvas", image));
        },
      }
    });

    match crate::OPTIONS.try_lock() {
      Ok(mut options) => {
        options.camera = self.ray_tracer.camera;
        options.rotation = self.ray_tracer.rotation;
        options.fov = self.ray_tracer.fov;
        options.width = self.ray_tracer.width;
        options.height = self.ray_tracer.height;
        options.scene = self.ray_tracer.scene.clone()
      },
      Err(_) => (),
    }

    ctx.request_repaint();
  }
}
