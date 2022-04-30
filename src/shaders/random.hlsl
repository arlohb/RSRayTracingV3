static float rng_seed = 42.;
static float2 rng_pixel = float2(0., 0.);

// https://stackoverflow.com/questions/5149544/can-i-generate-a-random-number-inside-a-pixel-shader
// I extended this a bit
float random_seed() {
  float2 p = rng_pixel + rng_seed;

  float2 K1 = float2(
    23.14069263277926, // e^pi (Gelfond's constant)
    2.665144142690225 // 2^sqrt(2) (Gelfondâ€“Schneider constant)
  );
  return rng_seed + frac( cos( dot(p, K1) ) * 12345.6789 ) * 1000.;
}

float random() {
  float2 p = rng_pixel + rng_seed;
  rng_seed = random_seed();

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

void random_init(float2 pixel) {
  rng_pixel = pixel;
  rng_seed = frame_data.jitter.x + frame_data.jitter.y;
  rng_seed = random_seed();
}
