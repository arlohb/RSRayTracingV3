use egui;
use std::sync::{Arc, Mutex};
use crate::ray_tracer::*;

fn vec3_widget(ui: &mut egui::Ui, vec3: &mut Vec3) {
  ui.horizontal(|ui| {
    ui.add(egui::DragValue::new(&mut vec3.x)
      .fixed_decimals(1)
      .speed(0.1));
    ui.add(egui::DragValue::new(&mut vec3.y)
      .fixed_decimals(1)
      .speed(0.1));
    ui.add(egui::DragValue::new(&mut vec3.z)
      .fixed_decimals(1)
      .speed(0.1));
  });
}

fn colour_widget(ui: &mut egui::Ui, input: &mut (f32, f32, f32)) {
  let mut colour = [input.0 as f32, input.1 as f32, input.2 as f32];
  ui.color_edit_button_rgb(&mut colour);
  *input = (colour[0] as f32, colour[1] as f32, colour[2] as f32);
}

fn data_row(ui: &mut egui::Ui, label: impl Into<egui::WidgetText>, widget: impl FnOnce(&mut egui::Ui)) {
  ui.columns(2, |ui| {
    ui[0].label(label);
    widget(&mut ui[1]);
  });
}

pub fn object_panel (ui: &mut egui::Ui, scene: &mut Scene) {
  ui.horizontal(|ui| {
    if ui.add(egui::Button::new("➕ sphere")).clicked() {
      scene.objects.push(Object {
        name: String::from("sphere"),
        material: Material {
          colour: (1., 0., 0.),
          emission: (0., 0., 0.),
          emission_strength: 0.,
          metallic: 0.5,
          roughness: 0.5,
        },
        geometry: Geometry::Sphere {
          center: Vec3 { x: 0., y: 0., z: 0., },
          radius: 1.,
        },
      });
    }
    if ui.add(egui::Button::new("➕ plane")).clicked() {
      scene.objects.push(Object {
        name: String::from("plane"),
        material: Material {
          colour: (1., 0., 0.),
          emission: (0., 0., 0.),
          emission_strength: 0.,
          metallic: 0.5,
          roughness: 0.5,
        },
        geometry: Geometry::Plane {
          center: Vec3 { x: 0., y: 0., z: 0., },
          normal: Vec3 { x: 0., y: 1., z: 0., },
          size: 5.,
        },
      });
    }

    if ui.add(egui::Button::new("JSON")).clicked() {
      crate::utils::log!("{}", serde_json::to_string_pretty(&scene).unwrap());
    }
  });

  ui.separator();

  ui.checkbox(&mut scene.do_objects_spin, "spin");

  ui.separator();

  egui::ScrollArea::vertical()
    .id_source("Objects")
    .always_show_scroll(true)
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
                  ui.add(egui::DragValue::new(radius)
                    .fixed_decimals(1)
                    .speed(0.1));
                });
              },
              Geometry::Plane { center: _, normal, size } => {
                data_row(ui, "normal", |ui| {
                  vec3_widget(ui, normal);
                  
                  *normal = normal.normalize();
                });

                data_row(ui, "size", |ui| {
                  ui.add(egui::DragValue::new(size)
                    .fixed_decimals(1)
                    .speed(0.1));
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
              ui.add(egui::DragValue::new(&mut object.material.emission_strength)
                .clamp_range::<f32>(0.0..=10.));
            });
            data_row(ui, "metallic", |ui| {
              ui.add(egui::DragValue::new(&mut object.material.metallic)
                .clamp_range::<f32>(0.0..=1.)
                .speed(0.1));
            });
            data_row(ui, "roughness", |ui| {
              ui.add(egui::DragValue::new(&mut object.material.roughness)
                .clamp_range::<f32>(0.0..=1.)
                .speed(0.1));
            });
          });
      };

      // padding so that the colour widget fits in the window
      for _ in 0..20 { ui.label(""); }
    });
}

pub fn settings_panel (
  ui: &mut egui::Ui,
  frame_times: Arc<Mutex<crate::utils::history::History>>,
  scene: &mut Scene,
) {
  ui.heading("Fps");

  let fps = 1000. / frame_times.lock().unwrap().average(None);
  let recent_fps = 1000. / frame_times.lock().unwrap().average(Some(1000.));

  data_row(ui, "5 sec average", |ui| {
    ui.label(format!("{:.1}", fps));
  });
  data_row(ui, "1 sec average", |ui| {
    ui.label(format!("{:.1}", recent_fps));
  });

  egui::plot::Plot::new("Fps history")
    .legend(egui::plot::Legend::default())
    .height(200.)
    .allow_zoom(false)
    .allow_drag(false)
    .include_y(0.)
    .show(ui, |ui| {
      ui.line(egui::plot::Line::new(egui::plot::Values::from_values(
        frame_times.lock().unwrap()
          .values(None)
          .iter()
          .map(|frame| egui::plot::Value::new(-frame.age(), 1000. / frame.value))
          .collect::<Vec<_>>()
        ))
          .name("Fps history")
      )
    });  

  ui.separator();

  data_row(ui, "position", |ui| {
    vec3_widget(ui, &mut scene.camera.position);
  });
  data_row(ui, "rotation", |ui| {
    vec3_widget(ui, &mut scene.camera.rotation);
  });

  data_row(ui, "bounces", |ui| {
    ui.add(egui::DragValue::new(&mut scene.reflection_limit)
      .clamp_range::<u32>(2..=10));
  });

  data_row(ui, "fov", |ui| {
    ui.add(egui::DragValue::new(&mut scene.camera.fov)
      .clamp_range::<f64>(1.0..=90.));
  });
}
