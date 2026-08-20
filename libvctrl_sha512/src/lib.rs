#![cfg_attr(not(test), no_std)]

#[macro_export]
macro_rules! impl_hmac {
    ($hash_struct:ty, $output_size:expr, $block_size:expr) => {
        #[doc = concat!("HMAC keyed-hash implementation using `", stringify!($hash_struct), "`.")]
        pub struct HMAC {
            ih: Option<$hash_struct>,
            padded: [u8; $block_size],
        }

        impl zeroize::Zeroize for HMAC {
            fn zeroize(&mut self) {
                if let Some(ref mut ih) = self.ih {
                    zeroize::Zeroize::zeroize(ih);
                }
                zeroize::Zeroize::zeroize(&mut self.padded);
            }
        }

        impl Drop for HMAC {
            fn drop(&mut self) {
                zeroize::Zeroize::zeroize(self);
            }
        }

        impl HMAC {
            fn prepare_key(k: &[u8]) -> [u8; $block_size] {
                let mut block_key = [0u8; $block_size];
                if k.len() > $block_size {
                    let hash = zeroize::Zeroizing::new(<$hash_struct>::hash(k));
                    let hash_bytes = &*hash;
                    block_key[..$output_size].copy_from_slice(&hash_bytes[..$output_size]);
                } else {
                    block_key[..k.len()].copy_from_slice(k);
                }
                block_key
            }

            #[doc = "One-shot HMAC computation."]
            #[must_use]
            pub fn mac<T: AsRef<[u8]>, U: AsRef<[u8]>>(input: T, k: U) -> [u8; $output_size] {
                let mut hmac = Self::new(k);
                hmac.update(input);
                hmac.finalize()
            }

            #[doc = "Creates a new HMAC context from a secret key."]
            #[must_use]
            pub fn new(k: impl AsRef<[u8]>) -> Self {
                let k = k.as_ref();
                let mut block_key = Self::prepare_key(k);
                let mut padded = [0x36u8; $block_size];
                for i in 0..$block_size {
                    padded[i] ^= block_key[i];
                }
                let mut ih = <$hash_struct>::new();
                ih.update(&padded);
                zeroize::Zeroize::zeroize(&mut block_key);
                HMAC {
                    ih: Some(ih),
                    padded,
                }
            }

            #[doc = "Feeds data into the HMAC."]
            pub fn update(&mut self, input: impl AsRef<[u8]>) {
                if let Some(ref mut ih) = self.ih {
                    ih.update(input);
                }
            }

            #[doc = "Finalizes the HMAC and returns the authentication tag."]
            #[must_use]
            pub fn finalize(mut self) -> [u8; $output_size] {
                for p in self.padded.iter_mut() {
                    *p ^= 0x6a;
                }
                let mut oh = <$hash_struct>::new();
                oh.update(&self.padded);
                let inner = zeroize::Zeroizing::new(
                    self.ih
                        .take()
                        .unwrap_or_else(|| <$hash_struct>::new())
                        .finalize(),
                );
                oh.update(&*inner);
                oh.finalize()
            }

            #[doc = "Finalizes the HMAC and verifies the tag against `expected` in constant-ish time."]
            #[inline]
            #[must_use]
            pub fn finalize_verify(self, expected: &[u8; $output_size]) -> bool {
                let out = self.finalize();
                $crate::utils::verify(&out, expected)
            }

            #[doc = "One-shot HMAC computation with verification."]
            #[inline]
            #[must_use]
            pub fn verify<T: AsRef<[u8]>, U: AsRef<[u8]>>(
                input: T,
                k: U,
                expected: &[u8; $output_size],
            ) -> bool {
                let mac = Self::mac(input, k);
                $crate::utils::verify(&mac, expected)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_hkdf {
    ($hash_struct:ty, $output_size:expr, $block_size:expr) => {
        #[doc = concat!("HKDF key derivation using `", stringify!($hash_struct), "`.")]
        pub struct HKDF;

        impl HKDF {
            #[doc = "HKDF-Extract step. Returns a pseudorandom key (PRK)."]
            #[inline]
            #[must_use]
            pub fn extract(salt: impl AsRef<[u8]>, ikm: impl AsRef<[u8]>) -> [u8; $output_size] {
                HMAC::mac(ikm, salt)
            }

            #[doc = "HKDF-Expand step. Fills `out` with output keying material."]
            #[inline]
            #[allow(clippy::arithmetic_side_effects, clippy::cast_possible_truncation)]
            pub fn expand(out: &mut [u8], prk: impl AsRef<[u8]>, info: impl AsRef<[u8]>) {
                assert_eq!(
                    prk.as_ref().len(),
                    $output_size,
                    "HKDF expects a {}-byte PRK",
                    $output_size
                );
                let info = info.as_ref();
                let max_blocks: u32 = 255;
                assert!(
                    (out.len() as u32) <= max_blocks * ($output_size as u32),
                    "Requested output exceeds RFC 5869 limit"
                );
                let mut offset = 0;
                let mut counter: u32 = 1;
                while offset < out.len() {
                    let mut hmac = HMAC::new(&prk);
                    if offset != 0 {
                        hmac.update(&out[offset - $output_size..][..$output_size]);
                    }
                    hmac.update(info);
                    hmac.update([counter as u8]);
                    let block = zeroize::Zeroizing::new(hmac.finalize());
                    let left = core::cmp::min($output_size, out.len() - offset);
                    out[offset..][..left].copy_from_slice(&block[..left]);
                    offset += $output_size;
                    counter = counter.wrapping_add(1);
                }
            }
        }
    };
}

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
