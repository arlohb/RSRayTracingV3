use std::sync::{Mutex, Arc};
use crate::{ray_tracer::*, panels::*, Time};

pub struct Ui {
  g_scene: Arc<Mutex<Scene>>,
  scene: Scene,
  last_time: f64,
  frame_times: Arc<Mutex<crate::History>>,
}

impl Ui {
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

  pub fn update(
    &mut self,
    ctx: &egui::Context,
    _: &epi::Frame,
    render_texture: crate::gpu::RenderTexture,
  ) {
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
      if let Some(id) = render_texture.id {
        ui.image(id, [render_texture.size.0 as f32, render_texture.size.1 as f32]);
      }
    });

    egui::SidePanel::right("panel").show(ctx, |ui| {
      ui.columns(2, |cols| {
        object_panel(&mut cols[0], &mut self.scene);
        settings_panel(&mut cols[1], self.frame_times.clone(), &mut self.scene);
      });
    });

    if let Ok(mut scene) = self.g_scene.try_lock() {
      *scene = self.scene.clone();
    }
  }
}
