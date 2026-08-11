//! SHA-512 hasher implementation for `libvctrl_core`.
//!
//! # Purpose
//! This module provides the [`Sha512Hasher`], a concrete implementation of the
//! [`Hasher`](libvctrl_handler::Hasher) trait. It bridges the pure-Rust,
//! `#![no_std]`-compatible [`libvctrl_sha512`] crate with the core version control
//! contracts defined in [`libvctrl_handler`].
//!
//! # Design rationale
//! - **Stateless and Zero-Cost**: The [`Sha512Hasher`] is a unit struct (Zero-Sized Type).
//!   It requires no heap allocations or internal state to be instantiated, making
//!   it extremely cheap to pass around or instantiate repeatedly.
//! - **Audited Cryptography**: By delegating the actual hashing to [`libvctrl_sha512`],
//!   this module ensures that the version control system relies on a carefully
//!   reviewed implementation of the SHA-512 algorithm, minimizing the attack surface
//!   for collision or preimage attacks.
//! - **Deterministic Addressing**: SHA-512 produces a 64-byte (512-bit) digest,
//!   which perfectly matches the [`HASH_LENGTH`](libvctrl_handler::HASH_LENGTH)
//!   constant defined in the contracts. This provides a massive keyspace, making
//!   accidental hash collisions practically impossible.
//!
//! # Internal mechanism
//! The hasher calls [`Sha512Hash::hash`](libvctrl_sha512::Hash::hash) on the input
//! data, which returns a statically sized `[u8; 64]` array. This array is then
//! converted into the canonical [`Hash`](libvctrl_handler::Hash) type using
//! [`Hash::from_bytes`](libvctrl_handler::Hash::from_bytes).
//! The conversion uses `.expect()` safely because the output length of SHA-512 is
//! statically guaranteed to be exactly 64 bytes by the algorithm's specification.

use libvctrl_handler::VctrlError;
use libvctrl_handler::{Hash, Hasher};
use libvctrl_sha512::Hash as Sha512Hash;

/// A cryptographic hasher that implements the SHA-512 algorithm.
///
/// # Purpose
/// This struct adapts the [`libvctrl_sha512`] crate to the
/// [`Hasher`](libvctrl_handler::Hasher) trait, allowing it to be used
/// transparently by the version control system to generate content-addressable
/// object identifiers.
///
/// # Design rationale
/// Because the underlying [`Sha512Hash`] uses static one-shot functions, this
/// adapter does not need to hold any state. It is a zero-sized type (ZST),
/// meaning it consumes no memory and can be copied freely.
///
/// # Examples
///
/// Hashing a simple byte string:
///
/// ```
/// use libvctrl_handler::Hasher;
/// use libvctrl_core::hash::Sha512Hasher;
///
/// let hasher = Sha512Hasher;
/// let hash = hasher.hash(b"hello world").unwrap();
/// assert_eq!(hash.as_bytes().len(), 64);
/// ```
#[derive(Debug, Default, Clone)]
pub struct Sha512Hasher;

impl Hasher for Sha512Hasher {
    /// Computes the SHA-512 digest of the provided data.
    ///
    /// # Design rationale
    /// This method takes `&self` to satisfy the trait interface, but since the
    /// hasher is stateless, it does not actually use `self`. It delegates
    /// directly to the one-shot [`Sha512Hash::hash`] function, ensuring optimal
    /// performance for one-off hashing tasks.
    ///
    /// # Internal mechanism
    /// 1. Calls `Sha512Hash::hash(data)` to get a `[u8; 64]` digest.
    /// 2. Wraps the digest in a [`Hash`] using `Hash::from_bytes`.
    /// 3. The `.expect()` is safe here because SHA-512 is mathematically
    ///    guaranteed to output exactly 64 bytes, matching `HASH_LENGTH`.
    ///
    /// # Examples
    ///
    /// Hashing the same data twice yields the same result, while different data
    /// yields different results:
    ///
    /// ```
    /// use libvctrl_handler::Hasher;
    /// use libvctrl_core::hash::Sha512Hasher;
    ///
    /// let hasher = Sha512Hasher;
    /// let hash1 = hasher.hash(b"data");
    /// let hash2 = hasher.hash(b"data");
    /// let hash3 = hasher.hash(b"other data");
    ///
    /// assert_eq!(hash1, hash2);
    /// assert_ne!(hash1, hash3);
    /// ```
    fn hash(&self, data: &[u8]) -> Result<Hash, VctrlError> {
        let digest = Sha512Hash::hash(data);
        Ok(Hash::from_bytes(&digest).unwrap())
    }
}
