#![no_std]
#![allow(
    non_snake_case,
    clippy::cast_lossless,
    clippy::eq_op,
    clippy::identity_op,
    clippy::many_single_char_names,
    clippy::unreadable_literal,
    clippy::cargo_common_metadata
)]
//! # libvctrl_sha512
//!
//! A **self‑contained**, **zero‑dependency**, `#![no_std]` Rust implementation of the
//! SHA‑512 cryptographic hash function, the HMAC‑SHA‑512 message authentication code,
//! and the HKDF‑SHA‑512 key derivation function (RFC 5869). An optional `sha384`
//! feature adds SHA‑384, HMAC‑SHA‑384, and HKDF‑SHA‑384.
//!
//! This crate is built as a trusted foundation for cryptographic operations in
//! resource‑constrained or bare‑metal environments, as well as in standard
//! applications that require **auditable**, **minimal dependency** code.
//!
//! ## Compliance & Auditing
//!
//! The implementation has undergone a security audit (v0.2.0) with all findings
//! addressed in this release. Key improvements over the original fork include:
//!
//! - **FIPS 180‑4 compliant message padding** – the 128‑bit big‑endian length is
//!   written in full, with the upper 64 bits zeroed, ensuring correctness for
//!   messages up to 2<sup>128</sup>‑1 bits (well beyond any practical limit).
//! - **PRK length validation** – `HKDF::expand` asserts that the supplied PRK
//!   is exactly the required length (64 bytes for SHA‑512, 48 bytes for SHA‑384),
//!   preventing silent misuse.
//! - **Edition 2021** – the crate uses a stable Rust edition, guaranteeing
//!   compatibility with current toolchains.
//! - **Idiomatic endian‑handling** – `load_be`/`store_be` use the built‑in
//!   `u64::from_be_bytes`/`to_be_bytes`, improving readability and optimizer
//!   friendliness.
//! - **Memory zeroisation** – HMAC instances wipe padded keys from the stack on
//!   drop, and the one‑shot `mac` function clears temporary key material.
//!
//! All cryptographic primitives produce output that matches the standard test
//! vectors (e.g., RFC 4231, RFC 5869, FIPS 180‑4 examples).
//!
//! ## Core components
//!
//! ### SHA‑512
//! The [`sha512`] module provides the fundamental hash function. It supports:
//! - **One‑shot hashing** via `Hash::hash(data) -> [u8; 64]`
//! - **Streaming hashing** with `Hash::new()`, `update()`, `finalize()`
//! - **Constant‑time verification** with `Hash::verify(expected)`
//!
//! ### HMAC‑SHA‑512
//! The [`hmac`] module implements RFC 2104 using SHA‑512. All MAC comparisons
//! use a timing‑attack resistant equality check. The API includes:
//! - `HMAC::mac(data, key)` for one‑shot MAC generation.
//! - `HMAC::new(key)` / `update()` / `finalize()` for incremental processing.
//! - `HMAC::verify(data, key, expected)` and `finalize_verify(expected)` for
//!   constant‑time verification.
//!
//! ### HKDF‑SHA‑512
//! The [`hkdf`] module implements the HMAC‑based Key Derivation Function (RFC 5869).
//! It follows the **extract‑then‑expand** paradigm:
//! - `HKDF::extract(salt, ikm) -> [u8; 64]` produces a pseudorandom key (PRK).
//! - `HKDF::expand(out, prk, info)` expands the PRK into arbitrary‑length output
//!   keying material (OKM).
//!
//! ### SHA‑384 (optional)
//! When the `sha384` feature is enabled (default), the [`sha384`] module provides
//! analogous types for SHA‑384, HMAC‑SHA‑384, and HKDF‑SHA‑384. These share the
//! same underlying compression function as SHA‑512 but differ in the initial vector
//! and output size (48 bytes).
//!
//! ## Design principles
//!
//! - **No standard library** – the crate only uses `core`, making it suitable for
//!   embedded systems, kernels, and WebAssembly. It avoids heap allocation entirely.
//! - **Minimal trusted code base** – no external dependencies beyond `core`,
//!   reducing supply‑chain risk and audit surface.
//! - **Constant‑time comparisons** – the [`utils::verify`] function compares byte
//!   slices in a data‑independent loop with a volatile read barrier, preventing
//!   compiler optimizations that could leak timing information. This is used for
//!   all MAC and hash verifications.
//! - **Zeroisation** – material derived from secret keys is explicitly overwritten
//!   when no longer needed. `HMAC::mac` zeroises its temporary buffers; `Drop`
//!   implementations clear the padded key.
//! - **Thread safety** – the `Hash` and `HMAC` types are `Copy` (or consume on
//!   finalize) and contain no interior mutability; they are safe to move between
//!   threads.
//!
//! ## Quick start examples
//!
//! ### SHA‑512 hashing
//! ```rust
//! use libvctrl_sha512::Hash;
//!
//! // One‑shot
//! let digest = Hash::hash(b"hello world");
//! assert_eq!(digest.len(), 64);
//!
//! // Streaming
//! let mut hasher = Hash::new();
//! hasher.update(b"hello ");
//! hasher.update(b"world");
//! let d = hasher.finalize();
//! assert_eq!(d, digest);
//!
//! // Constant‑time verification
//! let mut verifier = Hash::new();
//! verifier.update(b"hello world");
//! assert!(verifier.verify(&digest));
//! ```
//!
//! ### HMAC‑SHA‑512
//! ```rust
//! use libvctrl_sha512::HMAC;
//!
//! let mac = HMAC::mac(b"message", b"secret-key");
//! assert!(HMAC::verify(b"message", b"secret-key", &mac));
//!
//! // Streaming
//! let mut hmac = HMAC::new(b"secret-key");
//! hmac.update(b"first part ");
//! hmac.update(b"second part");
//! let m = hmac.finalize();
//! ```
//!
//! ### HKDF‑SHA‑512
//! ```rust
//! use libvctrl_sha512::HKDF;
//!
//! let ikm = b"input-key-material";
//! let salt = b"optional-salt";
//! let info = b"context-info";
//!
//! let prk = HKDF::extract(salt, ikm);
//! let mut out = [0u8; 32];
//! HKDF::expand(&mut out, prk, info);
//! // `out` is now 32 bytes of derived key material.
//! ```
//!
//! ## Feature flags
//!
//! - **`sha384`** *(enabled by default)* – activates the `sha384` module
//!   containing SHA‑384, HMAC‑SHA‑384, and HKDF‑SHA‑384.
//! - **`opt_size`** – trades some performance for reduced binary size. The
//!   `expand` and compression functions are marked `inline(never)`, yielding
//!   roughly 75% smaller code at a ~16% speed penalty.
//!
//! ## Minimum Supported Rust Version
//!
//! This crate requires Rust **1.70** or later. The `edition = "2021"` setting
//! ensures compatibility with current stable releases.
//!
//! ## License
//!
//! Licensed under the ISC License, the same as the original
//! [hmac-sha512](https://github.com/jedisct1/rust-hmac-sha512) from which this
//! crate is derived.
//!
//! ## Acknowledgement
//!
//! This project is a fork and modularisation of Frank Denis's excellent
//! [`hmac-sha512`](https://github.com/jedisct1/rust-hmac-sha512) crate. All core
//! cryptographic logic originates from his work.

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
