#![allow(clippy::indexing_slicing)]
use crate::hmac::HMAC;

impl_hkdf!(crate::sha512::Hash, 64, 128);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_returns_64_bytes() {
        let prk = HKDF::extract(b"", b"");
        assert_eq!(prk.len(), 64);
    }

    #[test]
    fn test_expand_zero_output_does_not_panic() {
        let mut out = [];
        HKDF::expand(&mut out, [0_u8; 64], b"");
    }

    #[test]
    fn test_expand_different_info_produces_different_output() {
        let prk = [0x42_u8; 64];
        let mut out_a = [0_u8; 32];
        let mut out_b = [0_u8; 32];
        HKDF::expand(&mut out_a, prk, b"a");
        HKDF::expand(&mut out_b, prk, b"b");
        assert_ne!(out_a, out_b);
    }

    #[test]
    #[should_panic(expected = "HKDF expects a 64-byte PRK")]
    fn test_expand_wrong_prk_length_panics() {
        let mut out = [0_u8; 32];
        HKDF::expand(&mut out, [0_u8; 16], b"");
    }

    #[test]
    #[should_panic(expected = "Requested output exceeds RFC 5869 limit")]
    fn test_expand_output_too_large_panics() {
        let mut out = [0_u8; 16_321];
        HKDF::expand(&mut out, [0_u8; 64], b"");
    }
}
