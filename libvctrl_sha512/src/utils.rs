pub const BLOCKBYTES: usize = 128;
pub const BYTES: usize = 64;

#[inline]
#[must_use]
pub fn load_be(base: &[u8], offset: usize) -> u64 {
    let bytes: [u8; 8] = offset
        .checked_add(8)
        .and_then(|end| base.get(offset..end))
        .and_then(|slice| slice.try_into().ok())
        .unwrap_or([0_u8; 8]);
    u64::from_be_bytes(bytes)
}

#[inline]
pub fn store_be(base: &mut [u8], offset: usize, x: u64) {
    if let Some(end) = offset.checked_add(8)
        && let Some(dst) = base.get_mut(offset..end)
    {
        dst.copy_from_slice(&x.to_be_bytes());
    }
}

#[must_use]
pub fn verify(x: &[u8], y: &[u8]) -> bool {
    let mut diff: u32 = 0;

    #[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
    {
        let (mut hash_x, mut hash_y) = (0_u32, 0_u32);
        for (byte_x, byte_y) in x.iter().zip(y.iter()) {
            hash_x ^= (hash_x << 5).wrapping_add((hash_x >> 2) ^ u32::from(*byte_x));
            hash_y ^= (hash_y << 5).wrapping_add((hash_y >> 2) ^ u32::from(*byte_y));
        }
        diff |= hash_x ^ hash_y;
    }

    for (byte_x, byte_y) in x.iter().zip(y.iter()) {
        diff |= u32::from(byte_x ^ byte_y);
    }

    if x.len() != y.len() {
        diff |= 0xffff_ffff;
    }

    let diff = core::hint::black_box(diff);
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_be_valid() {
        let bytes = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        assert_eq!(load_be(&bytes, 0), 0x0102_0304_0506_0708);
    }

    #[test]
    fn test_load_be_out_of_bounds_returns_zero() {
        let bytes = [0x01, 0x02, 0x03];
        assert_eq!(load_be(&bytes, 0), 0);
        assert_eq!(load_be(&bytes, 4), 0);
    }

    #[test]
    fn test_store_be_writes_big_endian() {
        let mut bytes = [0_u8; 10];
        store_be(&mut bytes, 1, 0x0102_0304_0506_0708);
        assert_eq!(&bytes[0..1], &[0]);
        assert_eq!(&bytes[1..9], &[1, 2, 3, 4, 5, 6, 7, 8][..]);
        assert_eq!(&bytes[9..10], &[0]);
    }

    #[test]
    fn test_store_be_out_of_bounds_does_nothing() {
        let mut bytes = [0xAA; 8];
        store_be(&mut bytes, 1, 0x1122_3344_5566_7788);
        assert_eq!(bytes, [0xAA; 8]);
    }

    #[test]
    fn test_verify_equal_empty_slices() {
        assert!(verify(&[], &[]));
    }

    #[test]
    fn test_verify_equal_same_length() {
        let a = [1, 2, 3];
        let b = [1, 2, 3];
        assert!(verify(&a, &b));
    }

    #[test]
    fn test_verify_different_same_length() {
        assert!(!verify(&[1, 2, 3], &[1, 2, 4]));
    }

    #[test]
    fn test_verify_different_length() {
        assert!(!verify(&[1, 2, 3], &[1, 2]));
    }
}
