struct Material { // 20
  colour: vec3<f32>; // 12
  specular: f32; // 4
  metallic: f32; // 4
};

struct Geometry { // 56
  option: u32; // 4
  // 12
  center: vec3<f32>; // 16
  vec_data: vec3<f32>; // 12
  f32_data: f32; // 4
  data: array<u32, 2>; // 8
};

struct Object { // 96
  material: Material; // 20
  // 12
  geometry: Geometry; // 56
  // 8
};

struct Objects {
  objects: array<Object>;
};

struct Light {
  options: u32;
  colour: vec3<f32>;
  vec_data: vec3<f32>;
};

struct Lights {
  lights: array<Light>;
};

struct Config {
  position: vec3<f32>;
  forward: vec3<f32>;
  right: vec3<f32>;
  up: vec3<f32>;
  background_colour: vec3<f32>;
  ambient_light: vec3<f32>;
  fov: f32;
  reflection_limit: u32;
  width: u32;
  height: u32;
};

struct Ray {
  origin: vec3<f32>;
  direction: vec3<f32>;
};

let PI = 3.141592654;

[[group(0), binding(0)]]
var<storage, read> objects: Objects;

[[group(0), binding(1)]]
var<storage, read> lights: Lights;

[[group(0), binding(2)]]
var<storage, read> config: Config;

// only returns the smallest value
fn solve_quadratic(a: f32, b: f32, c: f32) -> f32 {
  let discriminant = pow(b, 2.) - (4. * a * c);

  if (discriminant < 0.) {
    return 10000000.;
  }

  // let plus = (-b + sqrt(discriminant)) / (2. * a);
  let minus = (-b - sqrt(discriminant)) / (2. * a);

  return minus;
}

fn object_normal(object: Object, point: vec3<f32>) -> vec3<f32> {
  if (object.geometry.option == 0u) {
    return normalize(point - object.geometry.center);
  } if (object.geometry.option == 1u) {
    return object.geometry.vec_data;
  }

  return vec3<f32>(0., 1., 0.);
}

fn ray_intersect(ray: Ray, point: ptr<function, vec3<f32> >) -> i32 {
  var closest_dst = 1000000.;
  var closest_point = vec3<f32>(0., 0., 0.);
  var closest_object_index = -1;

  for(var i = 0u; i < arrayLength(&objects.objects); i = i + 1u) {
    let object = objects.objects[i];

    if (object.geometry.option == 0u) {
      let new_origin = ray.origin - object.geometry.center;

      let a = 1.;
      let b = 2. * dot(ray.direction, new_origin);
      let c = dot(new_origin, new_origin) - pow(object.geometry.f32_data, 2.);

      let distance = solve_quadratic(a, b, c);

      if (distance < closest_dst) {
        if (distance < 0.000001) { continue; }

        closest_dst = distance;
        closest_point = ray.origin + (ray.direction * distance);
        closest_object_index = i32(i);
      }
    } else if (object.geometry.option == 1u) {
      let denominator = dot(ray.direction, object.geometry.vec_data);

      if (abs(denominator) < 0.000001) { continue; }

      let numerator = dot(object.geometry.center - ray.origin, object.geometry.vec_data);
      let distance = numerator / denominator;

      let hit_point = ray.origin + (ray.direction * distance);

      if (
        abs(hit_point.x - object.geometry.center.x) > object.geometry.f32_data ||
        abs(hit_point.y - object.geometry.center.y) > object.geometry.f32_data ||
        abs(hit_point.z - object.geometry.center.z) > object.geometry.f32_data
      ) {
        continue;
      }

      if (distance < closest_dst) {
        if (distance < 0.000001) { continue; }

        closest_dst = distance;
        closest_point = hit_point;
        closest_object_index = i32(i);
      }
    }
  }

  *point = closest_point;
  return closest_object_index;
}

// fn calculate_local_colour(point: vec3<f32>, normal: vec3<f32>, material: Material) {
//   // the brightness starts at the ambient light level
//   var brightness = config.ambient_light;

//   for (var i = 0u; i < arrayLength(&lights.lights); i++) {
//     let light = lights.lights[i];

    
//   }
// }

fn trace_ray(ray: Ray, depth: u32) -> vec3<f32> {
  var point = vec3<f32>(0., 0., 0.);

  let object_index = ray_intersect(ray, &point);

  if (object_index == -1) {
    return config.background_colour;
  }

  return objects.objects[object_index].material.colour;

  // // get the normal at the point of intersection
  // let normal = object_normal(objects.objects[object_index], point);

  // // get the local colour of the object
}

[[stage(fragment)]]
fn fs_main([[builtin(position)]] coord_in: vec4<f32>) -> [[location(0)]] vec4<f32> {

  // calculate the viewport dimensions
  let fov_rad = config.fov * PI / 180.;
  let half_width = tan(fov_rad / 2.);
  let half_height = half_width * f32(config.height) / f32(config.width);

  let center = config.position - config.forward;

  let left = center - (config.right * half_width);
  let right = center + (config.right * half_width);
  let bottom = center - (config.up * half_height);
  let top = center + (config.up * half_height);

  let top_left = left + top - center;
  let width = half_width * 2.;
  let height = half_height * 2.;

  // create ray
  let x = coord_in.x / f32(config.width);
  let y = coord_in.y / f32(config.height);
  let x_offset = config.right * (x * width);
  let y_offset = -config.up * (y * height);

  let pixel_world_space = top_left + x_offset + y_offset;

  let ray = Ray(config.position, normalize(pixel_world_space - config.position));

  // calculate the colour of the pixel
  let pixel = trace_ray(ray, 0u);

  return vec4<f32>(pixel, 1.);

  // let t = objects.objects[0].geometry.f32_data;
  // return vec4<f32>(t, t, t, 1.);

  // return vec4<f32>(coord_in.x / f32(config.width), coord_in.y / f32(config.height), 0.0, 1.0);
}
