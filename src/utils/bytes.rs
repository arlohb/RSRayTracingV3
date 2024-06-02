pub trait AsBytes<const B: usize> {
    #[must_use]
    fn as_bytes(&self) -> [u8; B];
}

impl AsBytes<12> for nalgebra::Vector3<f32> {
    fn as_bytes(&self) -> [u8; 12] {
        unsafe { std::mem::transmute(self.data) }
    }
}

impl AsBytes<8> for nalgebra::Vector2<f32> {
    fn as_bytes(&self) -> [u8; 8] {
        unsafe { std::mem::transmute(self.data) }
    }
}

#[must_use]
pub fn bytes_concat<'a, const N: usize>(iter: impl Iterator<Item = &'a [u8]>) -> [u8; N] {
    puffin::profile_function!();

    let mut bytes = [0u8; N];
    let mut offset = 0;

    for input in iter {
        unsafe {
            let dst = bytes.as_mut_ptr().byte_add(offset);
            std::ptr::copy(input.as_ptr(), dst, input.len());
            offset += input.len();
        }
    }

    bytes
}

#[must_use]
pub fn bytes_concat_owned<const N: usize, const M: usize>(
    iter: impl Iterator<Item = [u8; N]>,
) -> [u8; M] {
    puffin::profile_function!();

    let mut bytes = [0u8; M];

    for (offset, input) in iter.enumerate() {
        unsafe {
            let dst = bytes.as_mut_ptr().cast::<[u8; N]>().add(offset);
            std::ptr::write(dst, input);
        }
    }

    bytes
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
