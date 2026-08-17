//! # HKDF Key Derivation (SHA-512)
//!
//! This module provides the HMAC-based Extract-and-Expand Key Derivation
//! Function (HKDF) as specified in RFC 5869, instantiated with SHA-512 as
//! the underlying hash function.
//!
//! ## What is HKDF?
//!
//! HKDF is a cryptographic key derivation function that turns secret input
//! keying material (IKM) into cryptographically strong output keying material
//! (OKM). It consists of two steps:
//!
//! - **Extract**: concentrates the entropy from the IKM into a fixed-size
//!   pseudorandom key (PRK) using an HMAC with a salt.
//! - **Expand**: stretches the PRK into additional keys of arbitrary length
//!   using HMAC with an info parameter for domain separation.
//!
//! ## How this module works
//!
//! The [`impl_hkdf!`] macro is invoked with `crate::sha512::Hash`, an output
//! size of 64 bytes, and a block size of 128 bytes. The macro generates the
//! [`HKDF`] struct with two static methods:
//!
//! - [`HKDF::extract`]: performs the extract step and returns a 64-byte PRK.
//! - [`HKDF::expand`]: performs the expand step and fills a caller-provided
//!   output buffer with key material.
//!
//! Internally, both methods delegate to the HMAC implementation generated for
//! SHA-512 by the [`impl_hmac!`] macro.
//!
//! # Examples
//!
//! Derive 42 bytes of output keying material:
//!
//! ```
//! # use libvctrl_sha512::hkdf::HKDF;
//! let ikm = b"input key material";
//! let salt = b"salt";
//! let info = b"context";
//!
//! let prk = HKDF::extract(salt, ikm);
//! let mut okm = [0u8; 42];
//! HKDF::expand(&mut okm, prk, info);
//!
//! assert_eq!(okm.len(), 42);
//! ```

use crate::hmac::HMAC;

impl_hkdf!(crate::sha512::Hash, 64, 128);
