use eframe::egui;
use crate::ray_tracer::*;

fn vec3_widget(ui: &mut egui::Ui, label: impl Into<egui::WidgetText>, vec3: &mut Vec3) {
  ui.horizontal(|ui| {
    ui.label(label);

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

pub fn object_panel (ui: &mut egui::Ui, scene: &mut Scene) {
  ui.horizontal(|ui| {
    if ui.add(egui::Button::new("➕ sphere")).clicked() {
      scene.objects.push(Object {
        name: String::from("sphere"),
        material: Material {
          colour: (1., 0., 0.),
          specular: 500.,
          metallic: 0.5,
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
          specular: 500.,
          metallic: 0.5,
        },
        geometry: Geometry::Plane {
          center: Vec3 { x: 0., y: 0., z: 0., },
          normal: Vec3 { x: 0., y: 1., z: 0., },
          size: 5.,
        },
      });
    }
    if ui.add(egui::Button::new("print")).clicked() {
      println!("{:#?}", scene.objects);
    }

    if ui.add(egui::Button::new("JSON")).clicked() {
      println!("{}", serde_json::to_string_pretty(&scene).unwrap());
    }
  });

  ui.separator();

  ui.checkbox(&mut scene.do_objects_spin, "spin");

  ui.separator();

  let mut has_removed_object = false;

  for i in 0..scene.objects.len() {
    let index = if has_removed_object { i - 1 } else { i };

    ui.horizontal(|ui| {
      ui.label(&scene.objects[index].name);

      if ui.add(egui::Button::new("❌")).clicked() {
        scene.objects.remove(index);
        has_removed_object = true;
      }
    });

    vec3_widget(ui, "pos", scene.objects[index].geometry.position_as_mut());

    if has_removed_object {
      continue;
    }

    let object = &mut scene.objects[index];

    match &mut object.geometry {
      Geometry::Sphere { center: _, radius } => {
        ui.horizontal(|ui| {
          ui.label("radius");
          ui.add(egui::DragValue::new(radius)
            .fixed_decimals(1)
            .speed(0.1));
        });
      },
      Geometry::Plane { center: _, normal, size } => {
        ui.horizontal(|ui| {
          ui.label("normal");
          ui.add(egui::DragValue::new(&mut normal.x)
            .fixed_decimals(1)
            .speed(0.1));
          ui.add(egui::DragValue::new(&mut normal.y)
            .fixed_decimals(1)
            .speed(0.1));
          ui.add(egui::DragValue::new(&mut normal.z)
            .fixed_decimals(1)
            .speed(0.1));
          
          *normal = normal.normalize();
        });

        ui.horizontal(|ui| {
          ui.label("size");
          ui.add(egui::DragValue::new(size)
            .fixed_decimals(1)
            .speed(0.1));
        });
      }
    }

    ui.horizontal(|ui| {
      ui.label("col");

      let mut colour = [object.material.colour.0 as f32, object.material.colour.1 as f32, object.material.colour.2 as f32];

      ui.color_edit_button_rgb(&mut colour);

      object.material.colour = (colour[0] as f64, colour[1] as f64, colour[2] as f64);

      ui.label("spec");
      ui.add(egui::DragValue::new(&mut object.material.specular)
        .clamp_range::<f64>(0.0..=1000.));
      
      ui.label("met");
      ui.add(egui::DragValue::new(&mut object.material.metallic)
        .clamp_range::<f64>(0.0..=1.)
        .speed(0.1));
    });

    ui.separator();
  };
}

pub fn settings_panel (ui: &mut egui::Ui, fps: f32, ray_tracer: &mut RayTracer, has_size_changed: &mut bool) {
  ui.heading("Settings");

  // this isn't perfect as I'm not using a fixed width font
  // but it still looks better than nothing
  ui.label(format!("fps: {: >4}", fps.round()));

  ui.separator();

  ui.horizontal(|ui| {

    let mut new_width = ray_tracer.width;
    let mut new_height = ray_tracer.height;

    ui.label("width");
    ui.add(egui::DragValue::new(&mut new_width)
      .speed(20));
    ui.label("height");
    ui.add(egui::DragValue::new(&mut new_height)
      .speed(20));

    if new_width != ray_tracer.width || new_height != ray_tracer.height {
      *has_size_changed = true;
      ray_tracer.width = new_width;
      ray_tracer.height = new_height;
    }
  });

  ui.separator();

  vec3_widget(ui, "pos", &mut ray_tracer.camera);
  vec3_widget(ui, "rot", &mut ray_tracer.rotation);

  ui.separator();

  ui.horizontal(|ui| {
    ui.label("bounces");
    ui.add(egui::DragValue::new(&mut ray_tracer.scene.reflection_limit)
      .clamp_range::<u32>(0..=10));
  });
}
