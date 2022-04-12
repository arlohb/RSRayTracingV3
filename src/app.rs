use eframe::{egui, epi};
use std::sync::{Mutex, Arc};
use crate::{ray_tracer::*, panels::*, Time};

pub struct App {
  g_scene: Arc<Mutex<Scene>>,
  scene: Scene,
  last_time: f64,
  frame_times: Arc<Mutex<crate::History>>,
}

impl App {
  pub fn new(
    g_scene: Arc<Mutex<Scene>>,
    frame_times: Arc<Mutex<crate::History>>,
  ) -> Self {
    Self {
      g_scene: g_scene.clone(),
      scene: g_scene.lock().unwrap().clone(),
      last_time: Time::now_millis(),
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
    let now = Time::now_millis();
    // delta_time is in seconds
    let delta_time = (now - self.last_time) as f32 / 1000.;
    self.last_time = now;

    crate::movement::move_and_rotate(
      &ctx.input(),
      &mut self.scene.camera,
      delta_time * 1.5,
      delta_time * 20.,
      6.,
      0.4,
    );

    if self.scene.do_objects_spin {
      self.scene.objects.iter_mut().for_each(|object| {
        if !matches!(object.geometry, Geometry::Sphere { .. }) {
          return;
        }

        let position = object.geometry.position_as_mut();
        let length = position.length();

        let theta: f32 = 0.5 * std::f32::consts::PI * delta_time;

        *position = position.transform_point(Mat44::create_rotation(Axis::Y, theta));

        // fix rounding errors?
        *position *= length / position.length();
      });
    }

    egui::CentralPanel::default().show(ctx, |ui| {
      ui.columns(2, |cols| {
        object_panel(&mut cols[0], &mut self.scene);
        settings_panel(&mut cols[1], self.frame_times.clone(), &mut self.scene);
      });
    });

    match self.g_scene.try_lock() {
      Ok(mut scene) => {
        *scene = self.scene.clone();
      },
      Err(_) => {},
    }

    ctx.request_repaint();
  }
}
