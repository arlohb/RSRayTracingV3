pub trait AsBytes<const B: usize> {
    #[must_use]
    fn as_bytes(&self) -> [u8; B];
}

impl AsBytes<12> for nalgebra::Vector3<f32> {
    fn as_bytes(&self) -> [u8; 12] {
        unsafe { std::mem::transmute(self.data) }
    }
}

#[must_use]
pub fn bytes_concat_n<const N: usize>(array: &[&[u8]]) -> [u8; N] {
    puffin::profile_function!();

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

#[must_use]
pub fn bytes_concat_fixed_in_n_iter<const N: usize, const M: usize>(
    iter: impl Iterator<Item = [u8; N]>,
) -> [u8; M] {
    puffin::profile_function!();

    let mut bytes = [0u8; M];

    iter.flatten().enumerate().for_each(|(i, byte)| {
        bytes[i] = byte;
    });

    bytes
}

#[must_use]
pub fn tuple_bytes<const N: usize>(tuple: (f32, f32, f32)) -> [u8; N] {
    puffin::profile_function!();

    bytes_concat_n(&[
        &tuple.0.to_le_bytes(),
        &tuple.1.to_le_bytes(),
        &tuple.2.to_le_bytes(),
    ])
}

#[must_use]
pub fn print_bytes<const N: usize>(bytes: &[u8; N]) -> String {
    let mut s = String::new();

    for (i, byte) in bytes.iter().enumerate() {
        if i % 4 == 0 {
            s.push(' ');
        }
        if i % 16 == 0 {
            s.push('\n');
        }
        s.push_str(format!("{byte:02x} ").as_str());
    }

    s
}
