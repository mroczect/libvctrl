//! # SHA‑512 Hasher Implementation
//!
//! This module provides [`Sha512Hasher`], a concrete [`Hasher`] that produces
//! 64‑byte digests using the SHA‑512 algorithm from the [`libvctrl_sha512`] crate.
//!
//! ## Why SHA‑512?
//!
//! - **Strong security** – 256‑bit collision resistance and 512‑bit preimage
//!   resistance.
//! - **64‑bit friendly** – the compression function is highly optimised for
//!   modern 64‑bit processors.
//! - **Standardised** – FIPS 180‑4 compliant, widely trusted in version control
//!   systems, digital signatures, and key derivation.
//!
//! ## Usage
//!
//! The hasher is stateless and implements the [`Hasher`] trait, so it can be
//! used wherever a generic hasher is required.
//!
//! ```rust
//! use libvctrl_core::hash::Sha512Hasher;
//! use libvctrl_handler::Hasher;
//!
//! let hasher = Sha512Hasher;
//! let hash = hasher.hash(b"hello world");
//! assert_eq!(hash.as_bytes().len(), 64);
//! ```
//!
//! ## Compatibility with `libvctrl_sha512` v0.3.0
//!
//! This implementation calls [`libvctrl_sha512::Hash::hash`], which computes a
//! SHA‑512 digest and returns a `[u8; 64]`.  The API has been stable since v0.1.0
//! and is fully compatible with v0.3.0.
//!
//! ## Performance
//!
//! `Sha512Hasher` delegates directly to the optimised SHA‑512 routine in
//! `libvctrl_sha512`.  For benchmarks, see the `libvctrl_sha512` crate.
//!
//! ## Security
//!
//! The digest is produced by a FIPS‑compliant implementation.  No `unsafe` code
//! is used.  Constant‑time equality checking for hashes is provided by the
//! [`Hash`] type from `libvctrl_handler` and the [`verify`](libvctrl_sha512::utils::verify)
//! function.

use libvctrl_handler::{Hash, Hasher};
use libvctrl_sha512::Hash as Sha512Hash;

/// A [`Hasher`] implementation using SHA‑512.
///
/// This hasher is stateless (no interior state) and can be freely copied and
/// shared.  Each call to [`hash`](Hasher::hash) produces a new 64‑byte digest
/// independently.
///
/// # Example
///
/// ```rust
/// use libvctrl_core::hash::Sha512Hasher;
/// use libvctrl_handler::Hasher;
///
/// let hasher = Sha512Hasher;
/// let digest = hasher.hash(b"some data");
/// assert_eq!(digest.as_bytes().len(), 64);
/// ```
#[derive(Debug, Default, Clone)]
pub struct Sha512Hasher;

impl Hasher for Sha512Hasher {
    /// Computes the SHA‑512 hash of `data`.
    ///
    /// The returned [`Hash`] always has exactly 64 bytes.
    fn hash(&self, data: &[u8]) -> Hash {
        let digest = Sha512Hash::hash(data);
        Hash::from_bytes(&digest).expect("SHA-512 produces 64 bytes")
    }
}
