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

float3 object_normal(Object object, float3 position) {
  if (object.geometry.option == 0) {
    return normalize(position - object.geometry.center);
  } if (object.geometry.option == 1) {
    return object.geometry.vec_data;
  }

  return float3(0., 1., 0.);
}

void object_intersect(inout Hit hit, uint i, Ray ray) {
  Object object = objects[i];

  if (object.geometry.option == 0) {
    float3 new_origin = ray.origin - object.geometry.center;

    float a = 1.;
    float b = 2. * dot(ray.direction, new_origin);
    float c = dot(new_origin, new_origin) - pow(object.geometry.f32_data, 2.);

    float distance = solve_quadratic(a, b, c);
    
    if (distance < hit.distance) {
      if (distance < EPSILON) { return; }

      hit.distance = distance;
      hit.position = ray.origin + (ray.direction * distance);
      hit.object_index = (int)i;
    }
  } else if (object.geometry.option == 1) {
    float denominator = dot(ray.direction, object.geometry.vec_data);

    if (abs(denominator) < EPSILON) { return; }

    float numerator = dot(object.geometry.center - ray.origin, object.geometry.vec_data);
    float distance = numerator / denominator;

    float3 hit_point = ray.origin + (ray.direction * distance);

    if (
      abs(hit_point.x - object.geometry.center.x) > object.geometry.f32_data |
      abs(hit_point.y - object.geometry.center.y) > object.geometry.f32_data |
      abs(hit_point.z - object.geometry.center.z) > object.geometry.f32_data
    ) {
      return;
    }

    if (distance < hit.distance) {
      if (distance < EPSILON) { return; }

      hit.distance = distance;
      hit.position = hit_point;
      hit.object_index = (int)i;
    }
  }
}

Hit ray_intersect(Ray ray) {
  Hit hit = Hit(float3(0.), 1000000., float3(0.), -1);

  for (uint i = 0; i < object_count; i += 1) {
    object_intersect(hit, i, ray);
  }

  hit.normal = object_normal(objects[hit.object_index], hit.position);

  return hit;
}
