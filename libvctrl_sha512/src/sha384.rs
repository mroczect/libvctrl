#![allow(clippy::indexing_slicing)]
#![allow(clippy::arithmetic_side_effects)]

use crate::sha512::{Hash as Sha512Hash, State};
use crate::utils::load_be;

#[inline]
fn new_state() -> State {
    const IV: [u8; 64] = [
        0xcb, 0xbb, 0x9d, 0x5d, 0xc1, 0x05, 0x9e, 0xd8, 0x62, 0x9a, 0x29, 0x2a, 0x36, 0x7c, 0xd5,
        0x07, 0x91, 0x59, 0x01, 0x5a, 0x30, 0x70, 0xdd, 0x17, 0x15, 0x2f, 0xec, 0xd8, 0xf7, 0x0e,
        0x59, 0x39, 0x67, 0x33, 0x26, 0x67, 0xff, 0xc0, 0x0b, 0x31, 0x8e, 0xb4, 0x4a, 0x87, 0x68,
        0x58, 0x15, 0x11, 0xdb, 0x0c, 0x2e, 0x0d, 0x64, 0xf9, 0x8f, 0xa7, 0x47, 0xb5, 0x48, 0x1d,
        0xbe, 0xfa, 0x4f, 0xa4,
    ];
    let mut state = [0_u64; 8];
    for (index, word) in state.iter_mut().enumerate() {
        *word = load_be(&IV, index * 8);
    }
    State(state)
}

#[derive(Clone)]
pub struct Hash(Sha512Hash);

impl core::fmt::Debug for Hash {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Hash")
    }
}

impl Hash {
    #[must_use]
    pub fn new() -> Self {
        Self(Sha512Hash {
            state: new_state(),
            r: 0,
            w: [0_u8; 128],
            len: 0,
        })
    }

    pub(crate) fn update_inner<T: AsRef<[u8]>>(&mut self, input: T) {
        self.0.update_inner(input);
    }

    pub fn update<T: AsRef<[u8]>>(&mut self, input: T) {
        self.update_inner(input);
    }

    #[must_use]
    pub fn finalize(self) -> [u8; 48] {
        let mut out = [0_u8; 48];
        let full = zeroize::Zeroizing::new(self.0.finalize());
        out.copy_from_slice(&full[..48]);
        out
    }

    #[must_use]
    pub fn hash<T: AsRef<[u8]>>(input: T) -> [u8; 48] {
        let mut hasher = Self::new();
        hasher.update(input);
        hasher.finalize()
    }

    pub fn zeroize(&mut self) {
        zeroize::Zeroize::zeroize(self);
    }
}

impl zeroize::Zeroize for Hash {
    fn zeroize(&mut self) {
        zeroize::Zeroize::zeroize(&mut self.0);
    }
}

impl Default for Hash {
    fn default() -> Self {
        Self::new()
    }
}

impl_hmac!(Hash, 48, 128);
impl_hkdf!(Hash, 48, 128);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_empty_vector() {
        let expected: [u8; 48] = [
            0x38, 0xb0, 0x60, 0xa7, 0x51, 0xac, 0x96, 0x38, 0x4c, 0xd9, 0x32, 0x7e, 0xb1, 0xb1,
            0xe3, 0x6a, 0x21, 0xfd, 0xb7, 0x11, 0x14, 0xbe, 0x07, 0x43, 0x4c, 0x0c, 0xc7, 0xbf,
            0x63, 0xf6, 0xe1, 0xda, 0x27, 0x4e, 0xde, 0xbf, 0xe7, 0x6f, 0x65, 0xfb, 0xd5, 0x1a,
            0xd2, 0xf1, 0x48, 0x98, 0xb9, 0x5b,
        ];
        assert_eq!(Hash::hash(b""), expected);
    }

    #[test]
    fn test_hash_abc_vector() {
        let expected: [u8; 48] = [
            0xcb, 0x00, 0x75, 0x3f, 0x45, 0xa3, 0x5e, 0x8b, 0xb5, 0xa0, 0x3d, 0x69, 0x9a, 0xc6,
            0x50, 0x07, 0x27, 0x2c, 0x32, 0xab, 0x0e, 0xde, 0xd1, 0x63, 0x1a, 0x8b, 0x60, 0x5a,
            0x43, 0xff, 0x5b, 0xed, 0x80, 0x86, 0x07, 0x2b, 0xa1, 0xe7, 0xcc, 0x23, 0x58, 0xba,
            0xec, 0xa1, 0x34, 0xc8, 0x25, 0xa7,
        ];
        assert_eq!(Hash::hash(b"abc"), expected);
    }

    #[test]
    fn test_hmac_sha384_rfc4231_case1() {
        let key = [0x0b_u8; 20];
        let data = b"Hi There";
        let mac = HMAC::mac(data, key);
        let expected: [u8; 48] = [
            0xaf, 0xd0, 0x39, 0x44, 0xd8, 0x48, 0x95, 0x62, 0x6b, 0x08, 0x25, 0xf4, 0xab, 0x46,
            0x90, 0x7f, 0x15, 0xf9, 0xda, 0xdb, 0xe4, 0x10, 0x1e, 0xc6, 0x82, 0xaa, 0x03, 0x4c,
            0x7c, 0xeb, 0xc5, 0x9c, 0xfa, 0xea, 0x9e, 0xa9, 0x07, 0x6e, 0xde, 0x7f, 0x4a, 0xf1,
            0x52, 0xe8, 0xb2, 0xfa, 0x9c, 0xb6,
        ];
        assert_eq!(mac, expected);
    }

    #[test]
    fn test_hkdf_extract_and_expand_basic() {
        let prk = HKDF::extract(b"salt", b"ikm");
        assert_eq!(prk.len(), 48);

        let mut out_a = [0_u8; 16];
        let mut out_b = [0_u8; 16];
        HKDF::expand(&mut out_a, prk, b"info-a");
        HKDF::expand(&mut out_b, prk, b"info-b");
        assert_ne!(out_a, out_b);
    }

    #[test]
    #[should_panic(expected = "HKDF expects a 48-byte PRK")]
    fn test_hkdf_expand_wrong_prk_length_panics() {
        let mut out = [0_u8; 16];
        HKDF::expand(&mut out, [0_u8; 16], b"");
    }

    #[test]
    #[should_panic(expected = "Requested output exceeds RFC 5869 limit")]
    fn test_hkdf_expand_output_too_large_panics() {
        let mut out = [0_u8; 12_241];
        HKDF::expand(&mut out, [0_u8; 48], b"");
    }
}
