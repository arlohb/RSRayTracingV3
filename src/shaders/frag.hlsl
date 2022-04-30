#include "inputs.hlsl"
#include "utils.hlsl"
#include "random.hlsl"
#include "ray.hlsl"

float roughness_to_phong_alpha(float roughness) {
  float smoothness = 1. - roughness;
  return pow(1000., smoothness * smoothness);
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

float4 fs_main(float4 position : SV_POSITION) : SV_TARGET {
  inputs_init();

  float2 pixel = position.xy + frame_data.jitter;

  random_init(pixel);

  float2 coord = pixel / float2(config.width, config.height);

  Ray ray = create_ray(coord);
  float3 colour = trace_ray_with_reflections(ray);

  float3 previous = t_render.Sample(s_tex, coord);
  float opacity = 1. / (float)(frame_data.progressive_count + 1);
  float3 mixed = colour * opacity + previous * (1. - opacity);

  return float4(mixed, 1.);
}
