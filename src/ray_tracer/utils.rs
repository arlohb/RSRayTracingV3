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
