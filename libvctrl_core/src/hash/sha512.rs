//! # SHA‑512 Hasher Implementation
//!
//! This module provides [`Sha512Hasher`], a concrete [`Hasher`] that produces
//! 64‑byte digests using the SHA‑512 algorithm from the [`libvctrl_sha512`] crate.
//!
//! ## Why SHA‑512?
//!
//! SHA‑512 is the standard choice for `libvctrl` for several reasons:
//!
//! - **Strong security** – 256‑bit collision resistance and 512‑bit preimage
//!   resistance. Even against quantum adversaries (Grover’s algorithm), the
//!   effective security remains above 128 bits.
//! - **64‑bit friendly** – The compression function operates on 64‑bit words
//!   and is highly optimised for modern 64‑bit processors (x86‑64, ARM64).
//!   On these platforms, SHA‑512 is often faster than SHA‑256.
//! - **Standardised** – FIPS 180‑4 compliant. It is widely trusted in version
//!   control systems (Git), digital signatures, and key derivation (HKDF).
//! - **Zero‑dependency implementation** – The `libvctrl_sha512` crate is a
//!   pure‑Rust, `#![no_std]` implementation that has been audited. It adds
//!   minimal supply‑chain risk.
//!
//! ## Usage
//!
//! The hasher is stateless and implements the [`Hasher`] trait, so it can be
//! used wherever a generic hasher is required. Because it contains no state,
//! it can be freely copied and shared.
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
//! Hashing 1 KB of data takes a few microseconds on a modern x86‑64 CPU.
//! The `opt_size` feature in `libvctrl_sha512` can reduce binary size by about
//! 75% at a cost of roughly 16% lower throughput.
//!
//! ## Security
//!
//! The digest is produced by a FIPS‑compliant implementation.  No `unsafe` code
//! is used.  Constant‑time equality checking for hashes is provided by the
//! [`Hash`] type from `libvctrl_handler` and the [`verify`](libvctrl_sha512::utils::verify)
//! function.
//!
//! The `expect` call in [`hash`](Hasher::hash) will never panic because
//! SHA‑512 always produces exactly 64 bytes, a property verified by the
//! test vectors from FIPS 180‑4 and RFC 6234.

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
///
/// # Why no streaming API?
///
/// The [`Hasher`] trait is intentionally minimal: a single `hash` method
/// that takes a complete byte slice. This is sufficient for version control
/// objects, which are always fully loaded before hashing. If you need
/// streaming hashing (e.g., for large files), you can build your own wrapper
/// that uses the streaming API of `libvctrl_sha512::Hash` directly and then
/// converts the result to a [`Hash`].
#[derive(Debug, Default, Clone)]
pub struct Sha512Hasher;

impl Hasher for Sha512Hasher {
    /// Computes the SHA‑512 hash of `data`.
    ///
    /// The returned [`Hash`] always has exactly 64 bytes.
    ///
    /// # Panics
    /// This method uses `expect` when converting the raw digest into a `Hash`.
    /// The panic is **impossible** because SHA‑512 always produces exactly 64 bytes,
    /// a property verified by the test vectors from FIPS 180‑4 and RFC 6234.
    /// If this ever fails, it indicates a critical bug in the underlying
    /// SHA‑512 implementation.
    fn hash(&self, data: &[u8]) -> Hash {
        let digest = Sha512Hash::hash(data);
        Hash::from_bytes(&digest).expect("SHA-512 produces 64 bytes")
    }
}
