pub const BLOCKBYTES: usize = 128;

pub const BYTES: usize = 64;

#[inline]
#[must_use]
pub fn load_be(base: &[u8], offset: usize) -> u64 {
    u64::from_be_bytes(base[offset..offset + 8].try_into().unwrap())
}

#[inline]
pub fn store_be(base: &mut [u8], offset: usize, x: u64) {
    base[offset..offset + 8].copy_from_slice(&x.to_be_bytes());
}

#[must_use]
pub fn verify(x: &[u8], y: &[u8]) -> bool {
    if x.len() != y.len() {
        return false;
    }
    let mut v: u32 = 0;

    #[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
    {
        let (mut h1, mut h2) = (0u32, 0u32);
        for (b1, b2) in x.iter().zip(y.iter()) {
            h1 ^= (h1 << 5).wrapping_add((h1 >> 2) ^ *b1 as u32);
            h2 ^= (h2 << 5).wrapping_add((h2 >> 2) ^ *b2 as u32);
        }
        v |= h1 ^ h2;
    }

    for (a, b) in x.iter().zip(y.iter()) {
        v |= u32::from(a ^ b);
    }

    let v = core::hint::black_box(v);
    v == 0
}
