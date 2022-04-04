use eframe::{egui, epi};
use std::sync::{Mutex, Arc};
use crate::{ray_tracer::*, panels::*, Time};

pub struct App {
  renderer: Renderer,
  texture: Option<eframe::epaint::TextureHandle>,
  last_time: f64,
  g_renderer: Arc<Mutex<Renderer>>,
  image: Arc<Mutex<eframe::epaint::ColorImage>>,
  frame_times: Arc<Mutex<eframe::egui::util::History<f32>>>,
}

impl App {
  pub fn new(
    renderer: Arc<Mutex<Renderer>>,
    image: Arc<Mutex<eframe::epaint::ColorImage>>,
    frame_times: Arc<Mutex<eframe::egui::util::History<f32>>>,
  ) -> Self {
    Self {
      renderer: renderer.lock().unwrap().clone(),
      texture: None,
      last_time: Time::now_millis(),
      g_renderer: renderer.clone(),
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

    let now = Time::now_millis();
    // delta_time is in seconds
    let delta_time = (now - self.last_time) / 1000.;
    self.last_time = now;

    {
      let renderer = &mut self.renderer;

      crate::movement::move_and_rotate(
        &ctx.input(),
        &mut renderer.scene.camera,
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
      options.width = self.renderer.width;
      options.height = self.renderer.height;
      options.scene = self.renderer.scene.clone()
    }

    ctx.request_repaint();
  }
}
