use std::ops::{Add, Div};

use crate::ray_tracer::{Geometry, Object, Scene, Vec3};
use puffin::GlobalFrameView;

fn vec3_widget(ui: &mut egui::Ui, vec3: &mut Vec3) {
    ui.horizontal(|ui| {
        ui.add(
            egui::DragValue::new(&mut vec3.x)
                .fixed_decimals(1)
                .speed(0.1),
        );
        ui.add(
            egui::DragValue::new(&mut vec3.y)
                .fixed_decimals(1)
                .speed(0.1),
        );
        ui.add(
            egui::DragValue::new(&mut vec3.z)
                .fixed_decimals(1)
                .speed(0.1),
        );
    });
}

fn colour_widget(ui: &mut egui::Ui, input: &mut Vec3) {
    let mut colour = (*input).into();
    ui.color_edit_button_rgb(&mut colour);
    *input = colour.into();
}

fn data_row(
    ui: &mut egui::Ui,
    label: impl Into<egui::WidgetText>,
    widget: impl FnOnce(&mut egui::Ui),
) {
    ui.columns(2, |ui| {
        ui[0].label(label);
        widget(&mut ui[1]);
    });
}

/// The objects panel.
pub fn object_panel(ui: &mut egui::Ui, scene: &mut Scene) {
    puffin::profile_function!();

    ui.horizontal(|ui| {
        if ui.add(egui::Button::new("➕ sphere")).clicked() {
            scene.objects.push(Object::default_sphere());
        }
        if ui.add(egui::Button::new("➕ plane")).clicked() {
            scene.objects.push(Object::default_plane());
        }
    });

    ui.separator();

    ui.checkbox(&mut scene.do_objects_spin, "spin");

    ui.separator();

    egui::ScrollArea::vertical()
        .id_source("Objects")
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
        .show(ui, |ui| {
            let mut has_removed_object = false;

            for i in 0..scene.objects.len() {
                let index = if has_removed_object { i - 1 } else { i };
                let name = scene.objects[index].name.clone();
                egui::CollapsingHeader::new(&name)
                    // If one isn't open, size is incorrect
                    .default_open(index == 0)
                    .show(ui, |ui| {
                        data_row(ui, name, |ui| {
                            if ui.add(egui::Button::new("❌")).clicked() {
                                scene.objects.remove(index);
                                has_removed_object = true;
                            }
                        });

                        if has_removed_object {
                            return;
                        }

                        let object = &mut scene.objects[index];

                        data_row(ui, "position", |ui| {
                            vec3_widget(ui, object.geometry.position_as_mut());
                        });

                        match &mut object.geometry {
                            Geometry::Sphere { center: _, radius } => {
                                data_row(ui, "radius", |ui| {
                                    ui.add(
                                        egui::DragValue::new(radius).fixed_decimals(1).speed(0.1),
                                    );
                                });
                            }
                            Geometry::Plane {
                                center: _,
                                normal,
                                size,
                            } => {
                                data_row(ui, "normal", |ui| {
                                    vec3_widget(ui, normal);

                                    *normal = normal.normalize();
                                });

                                data_row(ui, "size", |ui| {
                                    ui.add(egui::DragValue::new(size).fixed_decimals(1).speed(0.1));
                                });
                            }
                        }

                        data_row(ui, "colour", |ui| {
                            colour_widget(ui, &mut object.material.colour);
                        });
                        data_row(ui, "emission", |ui| {
                            colour_widget(ui, &mut object.material.emission);
                        });
                        data_row(ui, "strength", |ui| {
                            ui.add(
                                egui::DragValue::new(&mut object.material.emission_strength)
                                    .clamp_range::<f32>(0.0..=10.),
                            );
                        });
                        data_row(ui, "metallic", |ui| {
                            ui.add(
                                egui::DragValue::new(&mut object.material.metallic)
                                    .clamp_range::<f32>(0.0..=1.)
                                    .speed(0.1),
                            );
                        });
                        data_row(ui, "roughness", |ui| {
                            ui.add(
                                egui::DragValue::new(&mut object.material.roughness)
                                    .clamp_range::<f32>(0.0..=1.)
                                    .speed(0.1),
                            );
                        });
                    });
            }

            // padding so that the colour widget fits in the window
            for _ in 0..20 {
                ui.label("");
            }
        });
}

fn average<T: Default + Add<Output = T> + Div<Output = T> + From<u32>>(
    iter: impl Iterator<Item = T>,
) -> T {
    let mut sum: T = T::default();
    let mut count = 0u32;

    for i in iter {
        sum = sum + i;
        count += 1;
    }

    if count == 0 {
        T::default()
    } else {
        sum / T::from(count)
    }
}

fn ns_to_fps(ns: i64) -> f64 {
    if ns == 0 {
        0.
    } else {
        1_000_000_000f64 / ns as f64
    }
}

/// The settings panel.
pub fn settings_panel(
    ui: &mut egui::Ui,
    global_frame_view: &GlobalFrameView,
    show_profiler: &mut bool,
    scene: &mut Scene,
) {
    puffin::profile_function!();

    ui.heading("Fps");

    let mut profiling = puffin::are_scopes_on();
    let profiling_old = profiling;
    ui.checkbox(&mut profiling, "Puffin profiling");
    if profiling != profiling_old {
        puffin::set_scopes_on(profiling);
    }

    ui.checkbox(show_profiler, "Show profiler");

    let frame_view = global_frame_view.lock();
    let frames = frame_view
        .latest_frames(5 /* secs */ * 1000 /* fps guess */)
        .into_iter()
        .map(|frame| (frame.clone(), puffin::now_ns() - frame.meta().range_ns.0))
        .filter(|(_, elapsed)| *elapsed < 5_000_000_000)
        .collect::<Vec<_>>();

    let fps_1s = ns_to_fps(average(
        frames
            .iter()
            .filter(|(_, elapsed)| *elapsed < 1_000_000_000)
            .map(|(frame, _)| frame.duration_ns()),
    ));
    let fps_5s = ns_to_fps(average(frames.iter().map(|(frame, _)| frame.duration_ns())));

    data_row(ui, "5 sec average", |ui| {
        ui.label(format!("{fps_5s:.1}"));
    });
    data_row(ui, "1 sec average", |ui| {
        ui.label(format!("{fps_1s:.1}"));
    });

    let plot_points = frames
        .iter()
        .map(|(frame, elapsed)| {
            [
                -elapsed as f64 / 1_000_000_000f64,
                1_000_000_000f64 / frame.duration_ns() as f64,
            ]
        })
        .collect::<Vec<_>>();
    drop(frame_view);

    egui_plot::Plot::new("Fps history")
        .x_axis_label("Seconds")
        .y_axis_label("Fps")
        .height(400.)
        .allow_zoom(false)
        .allow_drag(false)
        .include_y(0.)
        .show(ui, |ui| {
            ui.line(egui_plot::Line::new(plot_points).name("Fps history"));
        });

    ui.separator();

    data_row(ui, "position", |ui| {
        vec3_widget(ui, &mut scene.camera.position);
    });
    data_row(ui, "rotation", |ui| {
        vec3_widget(ui, &mut scene.camera.rotation);
    });

    data_row(ui, "bounces", |ui| {
        ui.add(egui::DragValue::new(&mut scene.reflection_limit).clamp_range::<u32>(2..=20));
    });

    data_row(ui, "fov", |ui| {
        ui.add(egui::DragValue::new(&mut scene.camera.fov).clamp_range::<f64>(1.0..=90.));
    });
}
