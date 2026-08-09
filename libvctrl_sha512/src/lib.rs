//! # `libvctrl_sha512`
//!
//! A zero-dependency, `no_std`-compatible implementation of SHA-512, HMAC-SHA-512,
//! HKDF-SHA-512, and (optionally) SHA-384.
//!
//! The crate is built for performance and minimal code size. All hash algorithms
//! are implemented with careful attention to FIPS 180-4 and RFC 2104/5869. The
//! API is designed for simplicity: one-shot convenience functions sit alongside
//! incremental builders.
//!
//! ## Features
//!
//! - **default**: `sha384` – Includes SHA-384 support.
//! - **sha384**: Enables the [`sha384`] module.
//! - **`opt_size`**: Favours smaller code size over speed by controlling inlining.
//!
//! ## Quick Start
//!
//! Compute a SHA-512 hash:
//!
//! ```
//! # use libvctrl_sha512::Hash;
//! let digest = Hash::hash(b"hello world");
//! assert_eq!(digest.len(), 64);
//! ```
//!
//! Create an HMAC-SHA-512:
//!
//! ```
//! # use libvctrl_sha512::HMAC;
//! let key = b"my secret";
//! let tag = HMAC::mac(b"message", key);
//! assert_eq!(tag.len(), 64);
//! ```
//!
//! Derive keys with HKDF-SHA-512:
//!
//! ```
//! # use libvctrl_sha512::HKDF;
//! let ikm = [0x0b; 22];
//! let salt = [0x01; 13];
//! let info = [0xf0; 10];
//! let prk = HKDF::extract(salt, ikm);
//! let mut okm = [0u8; 42];
//! HKDF::expand(&mut okm, prk, info);
//! ```

#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    unused_qualifications
)]
// Allow unsafe code only in specific, reviewed cases (utils::verify).
#![allow(unused_crate_dependencies)]

/// Constructs an HMAC implementation using a given hash function.
///
/// This macro is the backbone of HMAC instantiation in the crate. It creates a
/// public `HMAC` struct with methods for keyed hashing, including one-shot
/// [`HMAC::mac`], incremental update, and verification that uses a non‑short‑circuiting
/// comparison (see [`crate::utils::verify`]).
///
/// ## Parameters
///
/// - `$hash_struct` – The hash type (e.g. [`crate::sha512::Hash`]).
/// - `$output_size` – Byte length of the hash output.
/// - `$block_size` – Byte length of the hash block.
///
/// ## Design
///
/// The implementation follows RFC 2104. Keys longer than the block size are
/// hashed first; shorter keys are zero‑padded. The inner and outer pads use the
/// standard `0x36` / `0x5c` constants. When the `HMAC` value is dropped, all
/// internal state is zeroed to reduce the lifetime of sensitive data in memory.
///
/// ## Example
///
/// Invoked inside this crate to create HMAC-SHA-512:
///
/// ```
/// # // The macro is expanded at compile time; this doctest just demonstrates
/// # // the resulting public API.
/// # use libvctrl_sha512::HMAC;
/// let key = b"password";
/// let msg = b"data";
/// let tag = HMAC::mac(msg, key);
/// assert_eq!(tag.len(), 64);
/// ```
#[macro_export]
macro_rules! impl_hmac {
    ($hash_struct:ty, $output_size:expr, $block_size:expr) => {
        #[doc = concat!("HMAC keyed-hash implementation using `", stringify!($hash_struct), "`.")]
        pub struct HMAC {
            ih: Option<$hash_struct>,
            padded: [u8; $block_size],
        }

        impl Drop for HMAC {
            fn drop(&mut self) {
                if let Some(ref mut ih) = self.ih {
                    ih.zeroize();
                }
                self.padded.fill(0);
            }
        }

        impl HMAC {
            fn prepare_key(k: &[u8]) -> [u8; $block_size] {
                let mut block_key = [0u8; $block_size];
                if k.len() > $block_size {
                    let hash = <$hash_struct>::hash(k);
                    block_key[..$output_size].copy_from_slice(&hash[..$output_size]);
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
                block_key.fill(0);
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
                let inner = self.ih.take().unwrap().finalize();
                oh.update(&inner);
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

/// Constructs an HKDF implementation using a given hash function.
///
/// This macro creates a public `HKDF` struct with the two-step HKDF key derivation
/// procedure from RFC 5869: `extract` and `expand`.
///
/// ## Parameters
///
/// - `$hash_struct` – The hash type (e.g. [`crate::sha512::Hash`]).
/// - `$output_size` – Length in bytes of the hash output (and PRK).
/// - `$block_size` – Block size of the hash.
///
/// ## Constraints
///
/// - `extract` produces a PRK of exactly `$output_size` bytes.
/// - `expand` requires a PRK of that exact length; it will panic otherwise.
/// - The maximum output length is `255 * $output_size` bytes, as mandated by the RFC.
///
/// ## Example
///
/// Using the HKDF-SHA-512 instance generated by this macro:
///
/// ```
/// # use libvctrl_sha512::HKDF;
/// let ikm = [0x0b; 22];
/// let salt = [0x01; 13];
/// let info = [0xf0; 10];
/// let prk = HKDF::extract(salt, ikm);
/// let mut okm = [0u8; 42];
/// HKDF::expand(&mut okm, prk, info);
/// ```
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
            pub fn expand(out: &mut [u8], prk: impl AsRef<[u8]>, info: impl AsRef<[u8]>) {
                assert_eq!(
                    prk.as_ref().len(),
                    $output_size,
                    "HKDF expects a {}-byte PRK",
                    $output_size
                );
                let info = info.as_ref();
                let mut counter: u8 = 1;
                assert!(
                    out.len() < 0xff * $output_size,
                    "Requested output exceeds RFC 5869 limit"
                );
                let mut i = 0;
                while i < out.len() {
                    let mut hmac = HMAC::new(&prk);
                    if i != 0 {
                        hmac.update(&out[i - $output_size..][..$output_size]);
                    }
                    hmac.update(info);
                    hmac.update([counter]);
                    let left = core::cmp::min($output_size, out.len() - i);
                    out[i..][..left].copy_from_slice(&hmac.finalize()[..left]);
                    counter += 1;
                    i += $output_size;
                }
            }
        }
    };
}

/// HMAC-SHA-512 and HMAC-SHA-384 implementations.
///
/// Contains the HMAC struct generated by [`impl_hmac!`] for SHA-512, and when the
/// `sha384` feature is enabled, for SHA-384 as well.
pub mod hkdf;

/// HKDF-SHA-512 and HKDF-SHA-384 key derivation.
///
/// Contains the HKDF struct generated by [`impl_hkdf!`] for SHA-512, and for SHA-384
/// when the feature is active.
pub mod hmac;

/// SHA-512 hash function implementation.
pub mod sha512;

/// Low‑level utilities (endianness, constant‑ish verification).
pub mod utils;

/// SHA-384 hash function implementation (requires `sha384` feature).
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
