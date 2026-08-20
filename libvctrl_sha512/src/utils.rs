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
