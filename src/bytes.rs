//! Manage casting structs to bytes, and joining the byte arrays together.

use std::cmp::min;

/// Get the struct represented as bytes, packed with HLSL's rules.
/// `B` is the length of the byte array.
pub trait AsBytes<const B: usize> {
    /// Get the struct represented as bytes, packed with HLSL's rules.
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

/// Concatenate an iterator of byte arrays.
/// The byte arrays are taken in as references so can be of variable lengths.
/// `N` is the length of the output byte array.
///     - If the input length is bigger than `N`, the bytes past N will be ignored
///     - If the input length is smaller than `N`, the output is padded with 0
#[must_use]
pub fn bytes_concat<'a, const N: usize>(iter: impl Iterator<Item = &'a [u8]>) -> [u8; N] {
    puffin::profile_function!();

    let mut bytes = [0u8; N];
    let mut offset = 0;

    for input in iter {
        let len = min(input.len(), N.saturating_sub(offset));

        if len == 0 {
            return bytes;
        }

        unsafe {
            let dst = bytes.as_mut_ptr().byte_add(offset);
            std::ptr::copy(input.as_ptr(), dst, len);
        }
        offset += len;
    }

    bytes
}

/// Concatenate an iterator of byte arrays.
/// The byte arrays are taken in as owned arrays so all have the same size of `M`.
/// `N` is the length of the output byte array.
///     - If the input length is bigger than `N`, the bytes past N will be ignored
///     - If the input length is smaller than `N`, the output is padded with 0
///
/// # Panics
///
/// If N is not a multiple of M.
#[must_use]
pub fn bytes_concat_owned<const M: usize, const N: usize>(
    iter: impl Iterator<Item = [u8; M]>,
) -> [u8; N] {
    puffin::profile_function!();

    // Check N is a multiple of M
    // Unfortunately can't be done at compile time
    assert_eq!(N % M, 0);

    let mut bytes = [0u8; N];

    for (offset, input) in iter.take(N / M).enumerate() {
        unsafe {
            let dst = bytes.as_mut_ptr().cast::<[u8; M]>().add(offset);
            std::ptr::write(dst, input);
        }
    }

    bytes
}

/// Returns a string of the bytes in hexadecimal.
#[allow(dead_code)]
#[must_use]
pub fn show_bytes(bytes: &[u8]) -> String {
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
