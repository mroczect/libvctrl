pub const BLOCKBYTES: usize = 128;
pub const BYTES: usize = 64;

#[inline]
#[must_use]
pub fn load_be(base: &[u8], offset: usize) -> u64 {
    let bytes: [u8; 8] = offset
        .checked_add(8)
        .and_then(|end| base.get(offset..end))
        .and_then(|s| s.try_into().ok())
        .unwrap_or([0u8; 8]);
    u64::from_be_bytes(bytes)
}

#[inline]
pub fn store_be(base: &mut [u8], offset: usize, x: u64) {
    if let Some(end) = offset.checked_add(8) {
        if let Some(dst) = base.get_mut(offset..end) {
            dst.copy_from_slice(&x.to_be_bytes());
        }
    }
}

#[must_use]
pub fn verify(x: &[u8], y: &[u8]) -> bool {
    let mut v: u32 = 0;

    #[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
    {
        let (mut h1, mut h2) = (0u32, 0u32);
        for (b1, b2) in x.iter().zip(y.iter()) {
            h1 ^= (h1 << 5).wrapping_add((h1 >> 2) ^ u32::from(*b1));
            h2 ^= (h2 << 5).wrapping_add((h2 >> 2) ^ u32::from(*b2));
        }
        v |= h1 ^ h2;
    }

    for (a, b) in x.iter().zip(y.iter()) {
        v |= u32::from(a ^ b);
    }

    if x.len() != y.len() {
        v |= 0xffff_ffff;
    }

    let v = core::hint::black_box(v);
    v == 0
}
