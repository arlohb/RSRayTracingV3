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

struct FrameData {
  jitter: vec2<f32>;
  progressive_count: u32;
};

let PI = 3.141592654;
let EPSILON = 0.0001;
let BOUNCE_LIMIT = // bounce_limit //;
1u; ////

[[group(0), binding(0)]]
var<storage, read> objects: Objects;

[[group(0), binding(1)]]
var<storage, read> lights: Lights;

[[group(0), binding(2)]]
var<storage, read> config: Config;

[[group(0), binding(3)]]
var s_tex: sampler;

[[group(0), binding(4)]]
var t_hdri: texture_2d<f32>;

[[group(0), binding(5)]]
var t_render: texture_2d<f32>;

[[group(0), binding(6)]]
var<storage, read> frame_data: FrameData;

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
        if (distance < EPSILON) { continue; }

        closest_dst = distance;
        closest_point = ray.origin + (ray.direction * distance);
        closest_object_index = i32(i);
      }
    } else if (object.geometry.option == 1u) {
      let denominator = dot(ray.direction, object.geometry.vec_data);

      if (abs(denominator) < EPSILON) { continue; }

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
        if (distance < EPSILON) { continue; }

        closest_dst = distance;
        closest_point = hit_point;
        closest_object_index = i32(i);
      }
    }
  }

  *point = closest_point;
  return closest_object_index;
}

fn get_point_to_light(light: Light, point: vec3<f32>) -> vec3<f32> {
  if (light.options == 0u) {
    return -light.vec_data;
  } else {
    return point - light.vec_data;
  }
}

fn calculate_local_colour(point: vec3<f32>, object: Object) -> vec3<f32> {
  let normal = object_normal(object, point);

  // the brightness starts at the ambient light level
  var brightness = config.ambient_light;

  for (var i = 0u; i < arrayLength(&lights.lights); i = i + 1u) {
    let light = lights.lights[i];

    let point_to_light = get_point_to_light(light, point);

    let shadow_ray = Ray(point, normalize(point_to_light));

    // ignore this light if object is in shadow
    var temp = vec3<f32>(0., 0., 0.);
    if (ray_intersect(shadow_ray, &temp) != -1) {
      continue;
    }

    // calculate the intensity of the light depending on the angle
    let angle_intensity = clamp(dot(normal, point_to_light) / (length(normal) * length(point_to_light)), 0., 1.);

    // calculate the specular intensity
    let reflection_vector = reflect(shadow_ray.direction, normal);
    let camera_vector = config.position - point;
    let specular = pow(
      clamp(
        dot(reflection_vector, camera_vector) / (length(reflection_vector) * length(camera_vector)), 0., 1.,
      ),
      object.material.specular,
    );

    brightness = brightness + (light.colour * (angle_intensity + specular));
  }

  return brightness * object.material.colour;
}

fn trace_ray(ray: ptr<function, Ray>, metallic_stack: ptr<function, array<f32, BOUNCE_LIMIT> >, index: u32) -> vec3<f32> {
  var point = vec3<f32>(0., 0., 0.);
  let object_index = ray_intersect(*ray, &point);

  let u = 0.5 + (atan2((*ray).direction.x, (*ray).direction.z) / (2. * PI));
  let v = 0.5 + (asin(-(*ray).direction.y) / PI);

  let hdri = textureSample(t_hdri, s_tex, vec2<f32>(u, v)).xyz;

  if (object_index == -1) {
    (*metallic_stack)[index] = 0.;
    // return config.background_colour;

    return hdri;
  }

  let object = objects.objects[object_index];

  (*metallic_stack)[index] = object.material.metallic;

  if (object.material.metallic != 0.) {
    *ray = Ray(point, reflect((*ray).direction, object_normal(object, point)));
  }

  return calculate_local_colour(point, object);
}

fn colour_lerp(local: vec3<f32>, reflection: vec3<f32>, metallic: f32) -> vec3<f32> {
  return local * (1. - metallic) + reflection * metallic;
}

fn stack_collapse(
  metallic_stack: ptr<function, array<f32, BOUNCE_LIMIT> >,
  reflection_colour_stack: ptr<function, array<vec3<f32>, BOUNCE_LIMIT> >,
  i_collapsed: u32,
) -> vec3<f32> {
  for (var i = i32(i_collapsed) - 1; i >= 0; i = i - 1) {
    (*reflection_colour_stack)[i] = colour_lerp(
      (*reflection_colour_stack)[i],
      (*reflection_colour_stack)[i + 1],
      (*metallic_stack)[i],
    );
  }

  return (*reflection_colour_stack)[0];
}

fn trace_ray_with_reflections(ray: Ray) -> vec3<f32> {
  var metallic_stack = array<f32, BOUNCE_LIMIT>(// metallic_stack_values //);
  0.); ////
  var reflection_colour_stack = array<vec3<f32>, BOUNCE_LIMIT>(// reflection_colour_stack_values //);
  vec3<f32>(0., 0., 0.)); ////

  var reflection_ray = ray;

  for (var i = 0u; i < BOUNCE_LIMIT; i = i + 1u) {
    reflection_colour_stack[i] = trace_ray(&reflection_ray, &metallic_stack, i);

    if (i != BOUNCE_LIMIT - 1u && metallic_stack[i] == 0.) {
      if (i == 0u) {
        return reflection_colour_stack[i];
      } else {
        return stack_collapse(&metallic_stack, &reflection_colour_stack, i);
      }
    }
  }

  return stack_collapse(&metallic_stack, &reflection_colour_stack, BOUNCE_LIMIT - 1u);
}

fn create_ray(coord: vec2<f32>) -> Ray {
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
  let x_offset = config.right * (coord.x * width);
  let y_offset = -config.up * (coord.y * height);

  let pixel_world_space = top_left + x_offset + y_offset;

  return Ray(config.position, normalize(pixel_world_space - config.position));
}

[[stage(fragment)]]
fn fs_main([[builtin(position)]] coord_in: vec4<f32>) -> [[location(0)]] vec4<f32> {
  let x = coord_in.x + frame_data.jitter.x;
  let y = coord_in.y + frame_data.jitter.y;
  let coord = vec2<f32>(x / f32(config.width), y / f32(config.height));

  let ray = create_ray(coord);

  let pixel = trace_ray_with_reflections(ray);

  let previous = textureSample(t_render, s_tex, vec2<f32>(coord.x, coord.y)).xyz;

  let opacity = 1. / (f32(frame_data.progressive_count) + 1.);

  let mixed = pixel * opacity + previous * (1. - opacity);

  return vec4<f32>(mixed, 1.);
}
