struct Material { // 48
  colour: vec3<f32>; // 12
  // 4
  emission: vec3<f32>; // 12
  emission_strength: f32; // 4
  metallic: f32; // 4
  roughness: f32; // 4
  // 8
};

struct Geometry { // 56
  option: u32; // 4
  // 12
  center: vec3<f32>; // 16
  vec_data: vec3<f32>; // 12
  f32_data: f32; // 4
  data: array<u32, 2>; // 8
};

struct Object { // 112
  material: Material; // 48
  geometry: Geometry; // 64
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
  energy: vec3<f32>;
};

struct Hit {
  point: vec3<f32>;
  distance: f32;
  normal: vec3<f32>;
  object_index: i32;
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

var<private> p_seed: f32 = 42.;
var<private> p_pixel: vec2<f32> = vec2<f32>(0., 0.);

fn random() -> f32 {
  let result = fract(sin(p_seed / 100. * dot(p_pixel, vec2<f32>(12.9898, 78.233))) * 43758.5453);
  p_seed = p_seed + 1.;
  return result;
}

fn get_tangent_space(normal: vec3<f32>) -> mat3x3<f32> {
  var helper = normalize(vec3<f32>(1., 0., 0.));
  if (abs(normal.x) > 0.99) {
    helper = vec3<f32>(0., 0., 1.);
  }

  let tangent = normalize(cross(normal, helper));
  let binormal = normalize(cross(normal, tangent));

  return mat3x3<f32>(tangent, binormal, normal);
}

fn roughness_to_phong_alpha(roughness: f32) -> f32 {
  let smoothness = 1. - roughness;
  return pow(1000., smoothness * smoothness);
}

fn random_in_hemisphere(normal: vec3<f32>, phong_alpha: f32) -> vec3<f32> {
  let cos_theta = pow(random(), 1. / (phong_alpha + 1.));
  let sin_theta = sqrt(1. - (cos_theta * cos_theta));
  let phi = 2. * PI * random();
  let tangent_space_dir = vec3<f32>(
    cos(phi) * sin_theta,
    sin(phi) * sin_theta,
    cos_theta,
  );

  return get_tangent_space(normal) * tangent_space_dir;
}

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

fn ray_intersect(ray: Ray) -> Hit {
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

  let normal = object_normal(objects.objects[closest_object_index], closest_point);

  return Hit(closest_point, closest_dst, normal, closest_object_index);
}

fn get_point_to_light(light: Light, point: vec3<f32>) -> vec3<f32> {
  if (light.options == 0u) {
    return -light.vec_data;
  } else {
    return point - light.vec_data;
  }
}

fn shade(ray: ptr<function, Ray>, hit: Hit) -> vec3<f32> {
  let hit = ray_intersect(*ray);

  let hdri_raw = textureSample(t_hdri, s_tex, vec2<f32>(
    0.5 + (atan2((*ray).direction.x, (*ray).direction.z) / (2. * PI)),
    0.5 + (asin(-(*ray).direction.y) / PI),
  )).xyz;

  let hdri = hdri_raw;

  if (hit.object_index == -1) {
    (*ray).energy =vec3<f32>(0., 0., 0.);
    return hdri;
  }

  let specular = vec3<f32>(0.6, 0.6, 0.6);
  let material = objects.objects[hit.object_index].material;

  (*ray).origin = hit.point + hit.normal * 0.001;

  let reflection_ray = reflect((*ray).direction, hit.normal);
  let hemisphere_sample = random_in_hemisphere(reflection_ray, roughness_to_phong_alpha(material.roughness));
  // (*ray).direction = roughness * hemisphere_sample + (1. - roughness) * reflection_ray;
  (*ray).direction = hemisphere_sample;

  (*ray).energy = (*ray).energy * (material.colour * clamp(dot(hit.normal, (*ray).direction), 0., 1.));

  return material.emission * material.emission_strength;
}

fn trace_ray_with_reflections(ray: ptr<function, Ray>) -> vec3<f32> {
  var result = vec3<f32>(0., 0., 0.);

  for (var i = 0u; i < BOUNCE_LIMIT; i = i + 1u) {
    let hit = ray_intersect(*ray);
    result = result + (*ray).energy * shade(ray, hit);

    if (length((*ray).energy) < EPSILON) {
      break;
    }
  }

  return result;
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

  return Ray(config.position, normalize(pixel_world_space - config.position), vec3<f32>(1., 1., 1.));
}

[[stage(fragment)]]
fn fs_main([[builtin(position)]] coord_in: vec4<f32>) -> [[location(0)]] vec4<f32> {
  let x = coord_in.x + frame_data.jitter.x;
  let y = coord_in.y + frame_data.jitter.y;

  p_pixel = vec2<f32>(x, y);

  let coord = vec2<f32>(x / f32(config.width), y / f32(config.height));

  var ray = create_ray(coord);

  let pixel = trace_ray_with_reflections(&ray);

  let previous = textureSample(t_render, s_tex, vec2<f32>(coord.x, coord.y)).xyz;

  let opacity = 1. / (f32(frame_data.progressive_count) + 1.);

  let mixed = pixel * opacity + previous * (1. - opacity);

  return vec4<f32>(mixed, 1.);
  // let hemisphere_sample = random_in_hemisphere(vec3<f32>(0., 0., 1.));
  // return vec4<f32>(abs(hemisphere_sample), 1.);
}
