pub fn bytes_concat_n<const N: usize>(array: &[&[u8]]) -> [u8; N] {
  let mut bytes = [0u8; N];

  array
    .concat()
    .as_slice()
    .iter()
    .enumerate()
    .for_each(|(i, byte)| {
      bytes[i] = *byte;
    });

  bytes
}

pub fn bytes_concat_fixed_in_n<const N: usize, const M: usize>(array: &[[u8; N]]) -> [u8; M] {
  let mut bytes = [0u8; M];

  array
    .concat()
    .as_slice()
    .iter()
    .enumerate()
    .for_each(|(i, byte)| {
      bytes[i] = *byte;
    });

  bytes
}

pub fn tuple_bytes<const N: usize>(tuple: (f32, f32, f32)) -> [u8; N] {
  bytes_concat_n(&[
    &tuple.0.to_le_bytes(),
    &tuple.1.to_le_bytes(),
    &tuple.2.to_le_bytes(),
  ])
}

/// Solves a quadratic equation in the form ax^2 + bx + c = 0
/// 
/// Returns None if there is not real solution
/// 
/// Returns Some(x, x) if there is only one real solution
/// 
/// Otherwise will return Some((x1, x2)) where x1 > x2
pub fn solve_quadratic (a: f32, b: f32, c: f32) -> Option<(f32, f32)> {
  let discriminant = b.powi(2) - (4. * a * c);

  if discriminant < 0. {
    return None;
  }

  if discriminant == 0. {
    let solution = (-b) / (2. * a);
    return Some((solution, solution));
  }

  let plus = (-b + discriminant.sqrt()) / (2. * a);
  let minus = (-b - discriminant.sqrt()) / (2. * a);

  Some((plus, minus))
}
