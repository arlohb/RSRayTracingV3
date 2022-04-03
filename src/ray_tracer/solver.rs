pub fn solve_quadratic (a: f64, b: f64, c: f64) -> Option<(f64, f64)> {
  let determinant = b.powi(2) - (4. * a * c);

  if determinant < 0. {
    return None;
  }

  if determinant == 0. {
    let solution = (-b) / (2. * a);
    return Some((solution, solution));
  }

  let plus = (-b + determinant.sqrt()) / (2. * a);
  let minus = (-b - determinant.sqrt()) / (2. * a);

  Some((plus, minus))
}