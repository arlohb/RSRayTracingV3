static float rng_seed = 42.;
static float2 rng_pixel = float2(0., 0.);

float random() {
  float2 coord = (rng_pixel + rng_seed) % 1.0;

  int size = config.width * config.height;
  rng_seed += t_random.SampleLevel(s_tex, coord, 0).x * 532.3412;
  rng_seed %= size;
  return t_random.SampleLevel(s_tex, coord, 0).x;
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

float3 random_in_hemisphere(float3 normal, float roughness) {
  if (roughness == 0.) {
    return normal;
  }

  float smoothness = 1. - roughness;
  float phong_alpha = pow(1000., smoothness * smoothness);

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

void random_init(float2 pixel) {
  rng_pixel = pixel;
}
