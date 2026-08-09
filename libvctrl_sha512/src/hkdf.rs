//! HKDF-SHA-512 (HMAC-based Key Derivation Function with SHA-512).
//!
//! This module instantiates the crate-level [`impl_hkdf!`] macro to produce a
//! public [`HKDF`] struct with `extract` and `expand` methods as specified in
//! [RFC 5869]. HKDF is used to derive cryptographically strong keys from
//! initial keying material (IKM) and an optional salt and info string.
//!
//! # Generated API
//!
//! - [`HKDF::extract`] – Performs the HKDF-Extract step, producing a
//!   pseudorandom key (PRK) of 64 bytes.
//! - [`HKDF::expand`] – Performs the HKDF-Expand step, generating output keying
//!   material (OKM) of arbitrary length up to `255 * 64` bytes.
//!
//! # Constraints
//!
//! - The PRK provided to [`HKDF::expand`] must be exactly 64 bytes; otherwise
//!   the function will panic.
//! - The total output length must be less than `255 * 64 = 16320` bytes, the
//!   maximum allowed by the RFC for SHA-512.
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
//!     0x14, 0x81, 0x57, 0x93, 0x38, 0xda, 0x36, 0x2c, 0xb8, 0xd9, 0xf9, 0x25, 0xd7, 0xcb,
//! ];
//! assert_eq!(okm, expected);
//! ```
//!
//! [RFC 5869]: https://datatracker.ietf.org/doc/html/rfc5869
use crate::hmac::HMAC;

impl_hkdf!(crate::sha512::Hash, 64, 128);
