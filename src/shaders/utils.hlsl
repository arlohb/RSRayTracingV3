static float PI = 3.141592654;
static float EPSILON = 0.000001;

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
