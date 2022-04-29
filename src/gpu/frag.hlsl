struct Material { // 48
  float3 colour; // 12
  int _0;// 4
  float3 emission; // 12
  float emission_strength; // 4
  float metallic; // 4
  float roughness; // 4
  int _1[2]; // 8
};

struct Geometry { // 56
  uint option; // 4
  int _0[3]; // 12
  float3 center; // 16
  float3 vec_data; // 12
  float f32_data; // 4
  uint data[2]; // 8
};

struct Object { // 112
  Material material; // 48
  Geometry geometry; // 64
};

struct Light {
  uint options;
  int _0[3];
  float3 colour;
  int _1;
  float3 vec_data;
  int _2;
};

struct Config {
  float3 position;
  int _0;
  float3 forward;
  int _1;
  float3 right;
  int _2;
  float3 up;
  int _3;
  float3 background_colour;
  int _4;
  float3 ambient_light;
  float fov;
  uint reflection_limit;
  uint width;
  uint height;
};

struct Ray {
  float3 origin;
  float3 direction;
  float3 energy;
};

struct Hit {
  float3 position;
  float distance;
  float3 normal;
  int object_index;
};

struct FrameData {
  float2 jitter;
  uint progressive_count;
};


static float PI = 3.141592654;
static float EPSILON = 0.0001;


// difference between StructuredBuffer and ConstantBuffer
// https://www.gamedev.net/forums/topic/624529-structured-buffers-vs-constant-buffers/4937832/
// https://docs.microsoft.com/en-us/windows/win32/direct3d12/resource-binding-in-hlsl#constant-buffers

// I don't think I need explicit register bindings,
// but I'll leave it here for now.

// register(b0, space0) = binding 0, group 0

StructuredBuffer<Object> objects : register(b0);
StructuredBuffer<Light> lights : register(b1);
ConstantBuffer<Config> config : register(b2);
SamplerState s_tex : register(b3);
Texture2D<float4> t_hdri : register(b4);
Texture2D<float4> t_render : register(b5);
ConstantBuffer<FrameData> frame_data : register(b6);

static float s_seed = 42.;
static float2 s_pixel = float2(0., 0.);
static uint s_object_count;


// https://stackoverflow.com/questions/5149544/can-i-generate-a-random-number-inside-a-pixel-shader
// I extended this a bit
float random_seed() {
  float2 p = s_pixel + s_seed;

  float2 K1 = float2(
    23.14069263277926, // e^pi (Gelfond's constant)
    2.665144142690225 // 2^sqrt(2) (Gelfondâ€“Schneider constant)
  );
  return s_seed + frac( cos( dot(p, K1) ) * 12345.6789 ) * 1000.;
}

float random() {
  float2 p = s_pixel + s_seed;
  s_seed = random_seed();

  float2 K1 = float2(
    23.14069263277926, // e^pi (Gelfond's constant)
    2.665144142690225 // 2^sqrt(2) (Gelfondâ€“Schneider constant)
  );
  return frac( cos( dot(p, K1) ) * 12345.6789 );
}

float3x3 get_tangent_space(float3 normal) {
  float3 helper = float3(1., 0., 0.);
  if (abs(normal.x) > 0.99) {
    helper = float3(0., 0., 1.);
  }

  float3 tangent = normalize(cross(normal, helper));
  float3 binormal = normalize(cross(normal, tangent));

  return float3x3(tangent, binormal, normal);
}

float3 random_in_hemisphere(float3 normal, float phong_alpha) {
  float cos_theta = pow(random(), 1. / (phong_alpha + 1.));
  float sin_theta = sqrt(1. - (cos_theta * cos_theta));
  float phi = 2. * PI * random();
  float3 tangent_space_dir = float3(
    cos(phi) * sin_theta,
    sin(phi) * sin_theta,
    cos_theta
  );

  return get_tangent_space(normal) * tangent_space_dir;
}

float roughness_to_phong_alpha(float roughness) {
  float smoothness = 1. - roughness;
  return pow(1000., smoothness * smoothness);
}

// only returns the smallest value
float solve_quadratic(float a, float b, float c) {
  float discriminant = pow(b, 2.) - (4. * a * c);

  if (discriminant < 0.) {
    return 10000000.;
  }

  // float plus = (-b + sqrt(discriminant)) / (2. * a);
  float minus = (-b - sqrt(discriminant)) / (2. * a);

  return minus;
}

float3 object_normal(Object object, float3 position) {
  if (object.geometry.option == 0) {
    return normalize(position - object.geometry.center);
  } if (object.geometry.option == 1) {
    return object.geometry.vec_data;
  }

  return float3(0., 1., 0.);
}

Hit ray_intersect(Ray ray) {
  Hit hit = Hit(float3(0.), 1000000., float3(0.), -1);

  for (uint i = 0; i < s_object_count; i += 1) {
    Object object = objects[i];

    if (object.geometry.option == 0) {
      float3 new_origin = ray.origin - object.geometry.center;

      float a = 1.;
      float b = 2. * dot(ray.direction, new_origin);
      float c = dot(new_origin, new_origin) - pow(object.geometry.f32_data, 2.);

      float distance = solve_quadratic(a, b, c);
      
      if (distance < hit.distance) {
        if (distance < EPSILON) { continue; }

        hit.distance = distance;
        hit.position = ray.origin + (ray.direction * distance);
        hit.object_index = (int)i;
      }
    } else if (object.geometry.option == 1) {
      float denominator = dot(ray.direction, object.geometry.vec_data);

      if (abs(denominator) < EPSILON) { continue; }

      float numerator = dot(object.geometry.center - ray.origin, object.geometry.vec_data);
      float distance = numerator / denominator;

      float3 hit_point = ray.origin + (ray.direction * distance);

      if (
        abs(hit_point.x - object.geometry.center.x) > object.geometry.f32_data |
        abs(hit_point.y - object.geometry.center.y) > object.geometry.f32_data |
        abs(hit_point.z - object.geometry.center.z) > object.geometry.f32_data
      ) {
        continue;
      }

      if (distance < hit.distance) {
        if (distance < EPSILON) { continue; }

        hit.distance = distance;
        hit.position = hit_point;
        hit.object_index = (int)i;
      }
    }
  }

  hit.normal = object_normal(objects[hit.object_index], hit.position);

  return hit;
}

float3 shade(inout Ray ray, Hit hit) {
  // need to use SampleLevel not Sample because this is done conditionally
  float3 hdri = t_hdri.SampleLevel(s_tex, float2(
    0.5 + (atan2(ray.direction.x, ray.direction.z) / (2. * PI)),
    0.5 + (asin(-ray.direction.y) / PI)),
    0
  ).rgb;

  if (hit.object_index == -1) {
    ray.energy = float3(0.);
    return hdri;
  }

  Material material = objects[hit.object_index].material;

  ray.origin = hit.position + hit.normal * 0.001;

  float3 reflection_ray = reflect(ray.direction, hit.normal);
  float3 hemisphere_sample = random_in_hemisphere(reflection_ray, roughness_to_phong_alpha(material.roughness));
  ray.direction = hemisphere_sample;

  ray.energy *= (2. * material.colour * clamp(dot(hit.normal, ray.direction), 0., 1.));

  return material.emission * material.emission_strength;
}

float3 trace_ray_with_reflections(Ray rayin) {
  Ray ray = rayin;
  float3 result = float3(0.);

  for (uint i = 0; i < config.reflection_limit; i += 1) {
    Hit hit = ray_intersect(ray);
    result += ray.energy * shade(ray, hit);

    if (length(ray.energy) < EPSILON) {
      break;
    }
  }

  return result;
}

Ray create_ray(float2 coord) {
  // calculate the viewport dimensions
  float fov_rad = config.fov * PI / 180.;
  float half_width = tan(fov_rad / 2.);
  float half_height = half_width * (float)config.height / (float)config.width;

  float3 center = config.position - config.forward;

  float3 left = center - (config.right * half_width);
  float3 right = center + (config.right * half_width);
  float3 bottom = center - (config.up * half_height);
  float3 top = center + (config.up * half_height);

  float3 top_left = left + top - center;
  float width = half_width * 2.;
  float height = half_height * 2.;

  // create ray
  float3 x_offset = config.right * (coord.x * width);
  float3 y_offset = -config.up * (coord.y * height);

  float3 pixel_world_space = top_left + x_offset + y_offset;

  return Ray(config.position, normalize(pixel_world_space - config.position), float3(1.));
}

float4 fs_main(float4 position : SV_POSITION) : SV_TARGET {
  uint stride;
  objects.GetDimensions(s_object_count, stride);
  s_pixel = position.xy + frame_data.jitter;
  s_seed = frame_data.jitter.x + frame_data.jitter.y;
  s_seed = random_seed();

  float2 coord = s_pixel / float2(config.width, config.height);

  Ray ray = create_ray(coord);
  float3 colour = trace_ray_with_reflections(ray);

  float3 previous = t_render.Sample(s_tex, coord);
  float opacity = 1. / (float)(frame_data.progressive_count + 1);
  float3 mixed = colour * opacity + previous * (1. - opacity);

  return float4(mixed, 1.);
}
