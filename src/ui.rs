use puffin::GlobalFrameView;

use crate::{
    panels::{object_panel, settings_panel},
    ray_tracer::{Axis, Geometry, Mat44, Scene},
    utils::time::now_millis,
};

pub struct Ui {
    last_time: f64,
    global_frame_view: GlobalFrameView,
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            last_time: now_millis(),
            global_frame_view: GlobalFrameView::default(),
        }
    }
}

impl Ui {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(
        &mut self,
        ctx: &egui::Context,
        render_target: &mut crate::gpu::RenderTarget,
        device: &wgpu::Device,
        scene: &mut Scene,
    ) {
        puffin::profile_function!();

        let now = now_millis();
        // delta_time is in seconds
        let delta_time = (now - self.last_time) as f32 / 1000.;
        self.last_time = now;

        ctx.input(|input_state| {
            crate::movement::move_and_rotate(
                input_state,
                &mut scene.camera,
                delta_time * 1.5,
                delta_time * 20.,
                6.,
                0.4,
            );
        });

        if scene.do_objects_spin {
            let theta: f32 = 0.5 * std::f32::consts::PI * delta_time;
            let rotation = Mat44::create_rotation(Axis::Y, theta);

            scene.objects.iter_mut().for_each(|object| {
                if let Geometry::Sphere { .. } = object.geometry {
                } else {
                    return;
                }

                let position = object.geometry.position_as_mut();
                let length = position.length();

                *position = position.transform_point(rotation);

                // correct for rounding errors
                *position *= length / position.length();
            });
        }

        if puffin::are_scopes_on() {
            puffin_egui::profiler_window(ctx);
        }

        egui::SidePanel::right("panel").show(ctx, |ui| {
            ui.columns(2, |cols| {
                object_panel(&mut cols[0], scene);
                settings_panel(&mut cols[1], &self.global_frame_view, scene);
            });
        });

        if let Some(id) = render_target.id {
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::Resize::default()
                    .default_size([render_target.size.0 as f32, render_target.size.1 as f32])
                    .min_size([1., 1.])
                    .show(ui, |ui| {
                        let size = (ui.available_size().x as u32, ui.available_size().y as u32);
                        if size != render_target.size {
                            render_target.resize(device, size);
                        }

                        ui.image(egui::ImageSource::Texture(egui::load::SizedTexture {
                            id,
                            size: egui::Vec2::new(
                                render_target.size.0 as f32,
                                render_target.size.1 as f32,
                            ),
                        }));
                    });
            });
        }
    }
}
