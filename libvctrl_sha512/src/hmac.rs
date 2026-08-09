//! HMAC-SHA-512 (Hash-based Message Authentication Code with SHA-512).
//!
//! This module instantiates the crate-level [`impl_hmac!`] macro to produce a
//! fully functional HMAC implementation using SHA-512 as the underlying hash
//! function. The macro generates a public [`HMAC`] struct and its associated
//! methods, following the HMAC construction defined in [RFC 2104].
//!
//! # Generated API
//!
//! - [`HMAC::new`] – Creates a new HMAC context from a secret key.
//! - [`HMAC::update`] – Feeds input data into the HMAC.
//! - [`HMAC::finalize`] – Produces the 64-byte authentication tag.
//! - [`HMAC::mac`] – One-shot HMAC computation.
//! - [`HMAC::verify`] / [`HMAC::finalize_verify`] – Verification in constant-ish
//!   time (see [`crate::utils::verify`]).
//!
//! # Key Handling
//!
//! Keys longer than the SHA-512 block size (128 bytes) are first hashed;
//! shorter keys are zero-padded. This matches the RFC specification.
//!
//! # Examples
//!
//! Computing an HMAC-SHA-512 tag in one shot:
//!
//! ```
//! # use libvctrl_sha512::hmac::HMAC;
//! let key = b"super secret key";
//! let message = b"important message";
//! let tag = HMAC::mac(message, key);
//! assert_eq!(tag.len(), 64);
//! ```
//!
//! Incremental usage with verification:
//!
//! ```
//! # use libvctrl_sha512::hmac::HMAC;
//! let key = b"another key";
//! let message = b"data to authenticate";
//! let expected = HMAC::mac(message, key);
//!
//! let mut hmac = HMAC::new(key);
//! hmac.update(&message[..4]);
//! hmac.update(&message[4..]);
//! assert!(hmac.finalize_verify(&expected));
//! ```
//!
//! [RFC 2104]: https://datatracker.ietf.org/doc/html/rfc2104
use crate::sha512::Hash;

impl_hmac!(Hash, 64, 128);
