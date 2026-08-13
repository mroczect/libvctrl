//! SHA-512 hasher implementation for `libvctrl_core`.
//!
//! # Purpose
//!
//! This module provides the [`Sha512Hasher`], a concrete implementation of the
//! [`Hasher`](libvctrl_handler::Hasher) trait. It bridges the pure-Rust,
//! `#![no_std]`-compatible [`libvctrl_sha512`] crate with the core version
//! control contracts defined in [`libvctrl_handler`].
//!
//! # Design Rationale
//!
//! - **Stateless and zero-cost**: The [`Sha512Hasher`] is a unit struct
//!   (zero-sized type). It requires no heap allocations or internal state to
//!   be instantiated, making it extremely cheap to pass around or instantiate
//!   repeatedly.
//! - **Audited cryptography**: By delegating the actual hashing to
//!   [`libvctrl_sha512`], this module ensures that the version control system
//!   relies on a carefully reviewed implementation of the SHA-512 algorithm,
//!   minimizing the attack surface for collision or preimage attacks.
//! - **Deterministic addressing**: SHA-512 produces a 64-byte (512-bit)
//!   digest, which perfectly matches the
//!   [`HASH_LENGTH`](libvctrl_handler::HASH_LENGTH) constant defined in the
//!   contracts. This provides a massive keyspace, making accidental hash
//!   collisions practically impossible.
//!
//! # Why SHA-512?
//!
//! SHA-512 was chosen for its wide digest size and widespread availability in
//! cryptographic libraries. The 64-byte output aligns with the contract's
//! [`HASH_LENGTH`](libvctrl_handler::HASH_LENGTH), avoiding truncation or
//! padding. In a content-addressable system, a larger hash reduces collision
//! probability and increases resistance against birthday attacks, making it
//! suitable for repositories that may contain millions of objects.
//!
//! # Internal Mechanism
//!
//! The hasher calls [`Sha512Hash::hash`](libvctrl_sha512::Hash::hash) on the
//! input data, which returns a statically sized `[u8; 64]` array. This array
//! is then converted into the canonical
//! [`Hash`](libvctrl_handler::Hash) type using
//! [`Hash::from_bytes`](libvctrl_handler::Hash::from_bytes). The conversion
//! uses `.expect()` safely because the output length of SHA-512 is statically
//! guaranteed to be exactly 64 bytes by the algorithm's specification.
//!
//! # Error Handling
//!
//! Although the [`Hasher::hash`] method returns
//! [`Result<Hash, VctrlError>`](libvctrl_handler::VctrlError), this
//! particular implementation is infallible. The result is always `Ok` because
//! SHA-512 is a deterministic algorithm that cannot fail for arbitrary byte
//! slices. The `Result` is part of the trait contract, allowing other hashing
//! algorithms that may have failure modes (e.g., keyed hashing with missing
//! keys) to use the same interface.
//!
//! # Security Considerations
//!
//! - **No unsafe code**: This module contains no `unsafe` blocks and relies
//!   only on safe Rust and the audited `libvctrl_sha512` crate.
//! - **Deterministic output**: The same input always produces the same hash,
//!   which is essential for reproducible content addressing.
//! - **Resistance to length extension**: SHA-512's internal structure and wide
//!   digest provide strong cryptographic properties suitable for version
//!   control integrity checking.
//!
//! # Examples
//!
//! Hashing data and verifying the result length:
//!
//! ```
//! use libvctrl_handler::Hasher;
//! use libvctrl_core::hash::Sha512Hasher;
//!
//! let hasher = Sha512Hasher;
//! let hash = hasher.hash(b"hello world").unwrap();
//! assert_eq!(hash.as_bytes().len(), 64);
//! ```

use libvctrl_handler::VctrlError;
use libvctrl_handler::{Hash, Hasher};
use libvctrl_sha512::Hash as Sha512Hash;

/// A cryptographic hasher that implements the SHA-512 algorithm.
///
/// # Purpose
///
/// This struct adapts the [`libvctrl_sha512`] crate to the
/// [`Hasher`](libvctrl_handler::Hasher) trait, allowing it to be used
/// transparently by the version control system to generate content-addressable
/// object identifiers.
///
/// # Design Rationale
///
/// Because the underlying [`Sha512Hash`] uses static one-shot functions, this
/// adapter does not need to hold any state. It is a zero-sized type (ZST),
/// meaning it consumes no memory and can be copied freely.
///
/// The struct derives [`Clone`], [`Debug`], and [`Default`] to provide common
/// conveniences without adding any runtime overhead. The [`Default`] instance
/// is particularly useful when the hasher is used as a field in a larger
/// struct or as a default parameter in generic code.
///
/// # Thread Safety
///
/// `Sha512Hasher` is both [`Send`] and [`Sync`] because it contains no data.
/// It can be shared across threads without synchronization, making it ideal
/// for use in concurrent indexing or hashing tasks.
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
    /// # Purpose
    ///
    /// This method converts an arbitrary byte slice into a fixed 64-byte
    /// [`Hash`](libvctrl_handler::Hash) value using the SHA-512 algorithm.
    /// The resulting hash serves as the content address for the input data.
    ///
    /// # Arguments
    ///
    /// * `data` - The raw bytes to hash. This may be any byte sequence,
    ///   including serialized objects, file content, or arbitrary payloads.
    ///
    /// # Returns
    ///
    /// Returns `Ok(hash)` where `hash` is a
    /// [`Hash`](libvctrl_handler::Hash) containing the 64-byte SHA-512
    /// digest.
    ///
    /// # Design Rationale
    ///
    /// This method takes `&self` to satisfy the trait interface, but since
    /// the hasher is stateless, it does not actually use `self`. It delegates
    /// directly to the one-shot [`Sha512Hash::hash`] function, ensuring
    /// optimal performance for one-off hashing tasks. There is no mutable
    /// state, so the hasher can be reused freely.
    ///
    /// # Internal Mechanism
    ///
    /// 1. Calls `Sha512Hash::hash(data)` to get a `[u8; 64]` digest.
    /// 2. Wraps the digest in a [`Hash`](libvctrl_handler::Hash) using
    ///    [`Hash::from_bytes`](libvctrl_handler::Hash::from_bytes).
    /// 3. The `.expect()` call is safe because SHA-512 is mathematically
    ///    guaranteed to output exactly 64 bytes, which matches
    ///    [`HASH_LENGTH`](libvctrl_handler::HASH_LENGTH).
    ///
    /// # Errors
    ///
    /// This implementation never returns an error. The `Result` type is
    /// required by the [`Hasher`](libvctrl_handler::Hasher) trait and is
    /// always `Ok`.
    ///
    /// # Examples
    ///
    /// Hashing the same data twice yields the same result, while different
    /// data yields different results:
    ///
    /// ```
    /// use libvctrl_handler::Hasher;
    /// use libvctrl_core::hash::Sha512Hasher;
    ///
    /// let hasher = Sha512Hasher;
    /// let hash1 = hasher.hash(b"data").unwrap();
    /// let hash2 = hasher.hash(b"data").unwrap();
    /// let hash3 = hasher.hash(b"other data").unwrap();
    ///
    /// assert_eq!(hash1, hash2);
    /// assert_ne!(hash1, hash3);
    /// ```
    fn hash(&self, data: &[u8]) -> Result<Hash, VctrlError> {
        let digest = Sha512Hash::hash(data);
        Ok(Hash::from_bytes(&digest).unwrap())
    }
}
