//! Zero-dependency cryptographic primitives: SHA-512, HMAC-SHA512, HKDF-SHA512,
//! and optional SHA-384.
//!
//! # Why this crate exists
//!
//! `libvctrl_sha512` provides a pure Rust, `no_std`-compatible implementation
//! of several widely used cryptographic algorithms. It is designed to serve as
//! the content-addressing and message-authentication backbone for the larger
//! `libvcrtl` version control system, while remaining usable as a standalone
//! cryptography crate.
//!
//! The implementation prioritizes:
//! - **Auditability** — no external dependencies and readable, well-structured code.
//! - **Security** — constant-time verification, zeroization of intermediate state.
//! - **Performance** — aggressive inlining, specialized block processing, and an
//!   optional `opt_size` feature for size-constrained builds.
//!
//! # Module organization
//!
//! - [`sha512`] — SHA-512 hash function.
//! - [`hmac`] — HMAC keyed-hash message authentication code instantiated with SHA-512.
//! - [`hkdf`] — HKDF key derivation function instantiated with SHA-512.
//! - [`utils`] — shared byte-order and verification helpers.
//! - [`sha384`] — optional SHA-384 implementation behind the `sha384` feature.
//!
//! The HMAC and HKDF modules are generated using the exported macros
//! [`impl_hmac!`] and [`impl_hkdf!`], which allow downstream crates to
//! instantiate these algorithms with other hash functions if needed.
//!
//! # Examples
//!
//! Compute a SHA-512 digest:
//!
//! ```
//! use libvctrl_sha512::Hash;
//!
//! let digest = Hash::hash(b"hello world");
//! assert_eq!(digest.len(), 64);
//! ```
//!
//! Compute an HMAC-SHA512 authentication tag:
//!
//! ```
//! use libvctrl_sha512::HMAC;
//!
//! let tag = HMAC::mac(b"message", b"secret-key");
//! assert_eq!(tag.len(), 64);
//! ```

/// Defines an HMAC (Hash-based Message Authentication Code) type based on the
/// provided hash struct.
///
/// # Why this macro exists
///
/// HMAC is a generic construction that can be built on top of any
/// cryptographic hash function. Rather than duplicating the implementation for
/// each hash algorithm, this macro generates a complete HMAC type from a hash
/// struct, output size, and block size. The generated type provides both
/// one-shot and incremental APIs.
///
/// # How it works
///
/// The macro expands to a struct named `HMAC` that wraps the chosen hash
/// implementation. It follows RFC 2104:
///
/// 1. Normalizes the key to the hash block size by hashing it if necessary.
/// 2. Computes the inner hash over the key XOR `0x36` and the message.
/// 3. Computes the outer hash over the key XOR `0x5c` and the inner digest.
///
/// The generated struct implements [`Drop`] to zeroize internal key material
/// and padded buffers when the context goes out of scope.
///
/// # Examples
///
/// The `libvctrl_sha512` crate already instantiates this macro for SHA-512:
///
/// ```
/// use libvctrl_sha512::HMAC;
///
/// let tag = HMAC::mac(b"message", b"key");
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

/// Defines an HKDF (HMAC-based Extract-and-Expand Key Derivation Function)
/// type based on the provided hash struct.
///
/// # Why this macro exists
///
/// HKDF is a key derivation function standardized in RFC 5869. It uses HMAC
/// internally and can be instantiated with any hash function that has an
/// associated HMAC implementation. This macro generates a complete `HKDF`
/// type from a hash struct, output size, and block size.
///
/// # How it works
///
/// The macro expands to a struct named `HKDF` with two associated functions:
///
/// - `extract` — computes a pseudorandom key (PRK) from the input key material
///   and an optional salt.
/// - `expand` — derives output keying material (OKM) of arbitrary length from
///   the PRK and optional context info.
///
/// The generated code enforces RFC 5869 limits on output length and PRK size.
///
/// # Examples
///
/// The `libvctrl_sha512` crate already instantiates this macro for SHA-512:
///
/// ```
/// use libvctrl_sha512::HKDF;
///
/// let prk = HKDF::extract(b"salt", b"input key material");
/// let mut okm = [0u8; 32];
/// HKDF::expand(&mut okm, prk, b"info");
/// assert_eq!(okm.len(), 32);
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

/// HMAC implementation generated for SHA-512.
///
/// This module contains the [`HMAC`](crate::HMAC) type, produced by the
/// [`impl_hmac!`] macro. It provides HMAC-SHA512 one-shot and incremental
/// authentication.
pub mod hmac;

/// HKDF implementation generated for SHA-512.
///
/// This module contains the [`HKDF`](crate::HKDF) type, produced by the
/// [`impl_hkdf!`] macro. It provides HKDF-SHA512 key derivation.
pub mod hkdf;

/// SHA-512 hash function implementation.
///
/// This module contains the [`Hash`](crate::Hash) type, which provides
/// incremental and one-shot SHA-512 hashing, along with verification and
/// zeroization support.
pub mod sha512;

/// Shared byte-order and verification helpers.
///
/// This module contains the [`load_be`](crate::utils::load_be),
/// [`store_be`](crate::utils::store_be), and
/// [`verify`](crate::utils::verify) functions, as well as the
/// [`BLOCKBYTES`](crate::utils::BLOCKBYTES) and
/// [`BYTES`](crate::utils::BYTES) constants.
pub mod utils;

/// Optional SHA-384 implementation.
///
/// This module is only available when the `sha384` feature is enabled. It
/// contains a SHA-384 hash type generated from the SHA-512 core.
#[cfg(feature = "sha384")]
pub mod sha384;

/// Re-export of the SHA-512 hash type.
///
/// This makes the primary hash type directly available as
/// `libvctrl_sha512::Hash`.
pub use sha512::Hash;

/// Re-export of the HMAC-SHA512 type.
///
/// This makes the HMAC type directly available as
/// `libvctrl_sha512::HMAC`.
pub use hmac::HMAC;

/// Re-export of the HKDF-SHA512 type.
///
/// This makes the HKDF type directly available as
/// `libvctrl_sha512::HKDF`.
pub use hkdf::HKDF;

/// Re-export of the SHA-512 utility constants.
///
/// This provides convenient access to [`BLOCKBYTES`](crate::utils::BLOCKBYTES)
/// and [`BYTES`](crate::utils::BYTES) at the crate root.
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
