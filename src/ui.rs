use std::sync::{Mutex, Arc};
use crate::{
  ray_tracer::*,
  panels::*,
  utils::time::now_millis,
};

pub struct Ui {
  g_scene: Arc<Mutex<Scene>>,
  scene: Scene,
  last_time: f64,
  frame_times: Arc<Mutex<crate::utils::history::History>>,
}

impl Ui {
  pub fn new(
    g_scene: Arc<Mutex<Scene>>,
    frame_times: Arc<Mutex<crate::utils::history::History>>,
  ) -> Self {
    Self {
      g_scene: g_scene.clone(),
      scene: g_scene.lock().unwrap().clone(),
      last_time: now_millis(),
      frame_times,
    }
  }

  pub fn update(
    &mut self,
    ctx: &egui::Context,
    _: &epi::Frame,
    render_target: &mut crate::gpu::RenderTarget,
  ) {
    let now = now_millis();
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

    if let Some(id) = render_target.id {
      egui::CentralPanel::default().show(ctx, |ui| {
        egui::Resize::default()
          .default_size([render_target.size.0 as f32, render_target.size.1 as f32])
          .min_size([1., 1.])
          .show(ui, |ui| {
            ui.image(id, [render_target.size.0 as f32, render_target.size.1 as f32]);   
       
            let size = (ui.available_size().x as u32, ui.available_size().y as u32);
            if size != render_target.size {
              
            }
          })
      });
    }

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
