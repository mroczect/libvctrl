pub mod hkdf;
pub mod hmac;
pub mod sha512;
pub mod utils;

#[cfg(feature = "sha384")]
pub mod sha384;

pub use hkdf::HKDF;
pub use hmac::HMAC;
pub use sha512::Hash;
pub use utils::{BLOCKBYTES, BYTES};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hmac_vectors() {
        let h = HMAC::mac([], [0u8; 32]);
        let expected: [u8; 64] = [
            185, 54, 206, 232, 108, 159, 135, 170, 93, 60, 111, 46, 132, 203, 90, 66, 57, 165, 254,
            80, 72, 10, 110, 198, 107, 112, 171, 91, 31, 74, 198, 115, 12, 108, 81, 84, 33, 179,
            39, 236, 29, 105, 64, 46, 83, 223, 180, 154, 215, 56, 30, 176, 103, 179, 56, 253, 123,
            12, 178, 34, 71, 34, 93, 71,
        ];
        assert_eq!(h, expected);
        assert!(HMAC::verify([], [0u8; 32], &expected));

        let h = HMAC::mac([42u8; 69], []);
        let expected: [u8; 64] = [
            56, 224, 189, 205, 65, 104, 107, 85, 241, 188, 253, 35, 238, 174, 69, 191, 206, 183,
            205, 71, 196, 180, 56, 122, 106, 55, 136, 7, 208, 183, 99, 67, 229, 213, 255, 154, 107,
            136, 11, 154, 11, 187, 75, 214, 172, 117, 14, 248, 189, 48, 193, 62, 37, 208, 159, 227,
            115, 59, 54, 91, 143, 143, 254, 220,
        ];
        assert_eq!(h, expected);
        assert!(HMAC::verify([42u8; 69], [], &expected));
    }

    #[test]
    fn hkdf_vector() {
        let ikm = [0x0bu8; 22];
        let salt: [u8; 13] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c,
        ];
        let info: [u8; 10] = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9];
        let expected: [u8; 42] = [
            0x83, 0x23, 0x90, 0x08, 0x6c, 0xda, 0x71, 0xfb, 0x47, 0x62, 0x5b, 0xb5, 0xce, 0xb1,
            0x68, 0xe4, 0xc8, 0xe2, 0x6a, 0x1a, 0x16, 0xed, 0x34, 0xd9, 0xfc, 0x7f, 0xe9, 0x2c,
            0x14, 0x81, 0x57, 0x93, 0x38, 0xda, 0x36, 0x2c, 0xb8, 0xd9, 0xf9, 0x25, 0xd7, 0xcb,
        ];
        let prk = HKDF::extract(salt, ikm);
        let mut okm = [0u8; 42];
        HKDF::expand(&mut okm, prk, info);
        assert_eq!(okm, expected);
    }
}
