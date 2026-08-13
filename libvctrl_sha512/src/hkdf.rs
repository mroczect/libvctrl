//! HKDF-SHA-512 (HMAC-based Key Derivation Function with SHA-512).
//!
//! # Purpose
//!
//! This module instantiates the crate-level [`impl_hkdf!`] macro to produce a
//! public [`HKDF`] struct with `extract` and `expand` methods as specified in
//! [RFC 5869]. HKDF is used to derive cryptographically strong keys from
//! initial keying material (IKM) and an optional salt and info string.
//!
//! # What is HKDF?
//!
//! HKDF is a two-phase key derivation function built on HMAC. It is designed
//! to take potentially weak or non-uniformly random input keying material
//! (IKM) and turn it into uniformly distributed output keying material (OKM)
//! suitable for use as cryptographic keys.
//!
//! The two phases are:
//!
//! 1. **HKDF-Extract**: Mixes the IKM with an optional salt to produce a
//!    pseudorandom key (PRK). The PRK has a fixed length equal to the hash
//!    output length, in this case 64 bytes.
//! 2. **HKDF-Expand**: Expands the PRK into any desired amount of output
//!    keying material using an optional `info` context string. The output is
//!    generated in blocks and is suitable for use as encryption keys,
//!    authentication keys, or other secrets.
//!
//! # Generated API
//!
//! - [`HKDF::extract`] - Performs the HKDF-Extract step, producing a
//!   pseudorandom key (PRK) of 64 bytes.
//! - [`HKDF::expand`] - Performs the HKDF-Expand step, generating output
//!   keying material (OKM) of arbitrary length up to `255 * 64` bytes.
//!
//! # Design Rationale
//!
//! This module does not implement HKDF manually. Instead, it invokes the
//! [`impl_hkdf!`] macro with:
//!
//! - [`crate::sha512::Hash`] as the underlying hash function.
//! - `64` as the hash output length.
//! - `128` as the SHA-512 block size.
//!
//! This macro-based design keeps the HKDF implementation generic across hash
//! functions. The same macro is reused for SHA-384 when the `sha384` feature
//! is enabled, with output length 48 and block size 128. The approach avoids
//! code duplication and ensures the implementation exactly matches the
//! HMAC-based RFC construction.
//!
//! # Constraints
//!
//! - The PRK provided to [`HKDF::expand`] must be exactly 64 bytes; otherwise
//!   the function will panic.
//! - The total output length must be less than `255 * 64 = 16320` bytes, the
//!   maximum allowed by the RFC for SHA-512.
//!
//! # Security Considerations
//!
//! - **Salt selection**: The salt is optional but recommended. A secret or
//!   random salt adds entropy and ensures that the same IKM produces different
//!   keys in different contexts. If no salt is provided, HKDF uses a string of
//!   zeros as required by RFC 5869.
//! - **Info string**: The `info` parameter binds the derived key to a specific
//!   context. It can be empty, but using a unique context string per key
//!   purpose prevents key reuse across different applications.
//! - **No unsafe code**: The implementation uses only safe Rust and the
//!   audited SHA-512/HMAC code in this crate.
//!
//! # Internal Mechanism
//!
//! [`HKDF::extract`] delegates to `HMAC::mac` with the salt as the HMAC key
//! and the IKM as the HMAC message. This is exactly the RFC 5869 extract
//! step:
//!
//! `PRK = HMAC-Hash(salt, IKM)`
//!
//! [`HKDF::expand`] iteratively computes HMAC blocks:
//!
//! `T(1) = HMAC-Hash(PRK, info || 0x01)`
//! `T(i) = HMAC-Hash(PRK, T(i-1) || info || i)`
//!
//! The first `N` bytes of the concatenated block stream are copied into the
//! output buffer. The loop in the macro uses a counter byte starting at 1 and
//! increments it after each block.
//!
//! # Examples
//!
//! Basic key derivation using the RFC 5869 test vector:
//!
//! ```
//! # use libvctrl_sha512::hkdf::HKDF;
//! let ikm = [0x0bu8; 22];
//! let salt: [u8; 13] = [
//!     0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c,
//! ];
//! let info: [u8; 10] = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9];
//!
//! let prk = HKDF::extract(salt, ikm);
//! let mut okm = [0u8; 42];
//! HKDF::expand(&mut okm, prk, info);
//!
//! let expected: [u8; 42] = [
//!     0x83, 0x23, 0x90, 0x08, 0x6c, 0xda, 0x71, 0xfb, 0x47, 0x62, 0x5b, 0xb5, 0xce, 0xb1,
//!     0x68, 0xe4, 0xc8, 0xe2, 0x6a, 0x1a, 0x16, 0xed, 0x34, 0xd9, 0xfc, 0x7f, 0xe9, 0x2c,
//!     0x14, 0x81, 0x57, 0x93, 0x38, 0xda, 0x36, 0x2c, 0xb8, 0xd9, 0xf9, 0x25, 0xd7, 0x0b,
//! ];
//! assert_eq!(okm, expected);
//! ```
//!
//! Deriving keys with no salt and empty info:
//!
//! ```
//! # use libvctrl_sha512::hkdf::HKDF;
//! let ikm = b"some weak input";
//! let prk = HKDF::extract([], ikm);
//! let mut okm = [0u8; 32];
//! HKDF::expand(&mut okm, prk, []);
//! assert_eq!(okm.len(), 32);
//! ```
//!
//! [RFC 5869]: https://datatracker.ietf.org/doc/html/rfc5869
use crate::hmac::HMAC;

impl_hkdf!(crate::sha512::Hash, 64, 128);
