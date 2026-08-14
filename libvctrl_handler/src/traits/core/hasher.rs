//! Cryptographic hashing to produce content addresses.
//!
//! # Purpose
//!
//! This module defines the `Hasher` trait, which abstracts the
//! content-addressing algorithm used to identify version control objects.
//! Content addressing is the cornerstone of the object model: every object
//! is identified by the cryptographic hash of its serialized bytes, ensuring
//! integrity and deduplication.
//!
//! # Design Rationale
//!
//! Hashing is separated into a trait for several reasons:
//!
//! - **Algorithm agility**: Different deployments may prefer SHA-256,
//!   SHA-512, BLAKE3, or other digest functions. The trait allows swapping
//!   the algorithm without touching storage or object logic.
//! - **Testability**: Dummy or deterministic hashers can be injected in unit
//!   tests, avoiding the need for actual cryptographic operations.
//! - **Decoupling**: The core data types remain independent of any specific
//!   hash implementation. Only the trait contract matters.
//!
//! # Why `Result`?
//!
//! Although most cryptographic hash functions are infallible for arbitrary
//! byte slices, the `hash` method returns `Result<Hash, VctrlError>`.
//! This design accounts for:
//!
//! - Hardware or library failures in exotic backends.
//! - Keyed hashing algorithms that may fail if no key is configured.
//! - Future extensions where hashing may involve fallible resources.
//!
//! The error type is `VctrlError`, preserving a unified
//! error surface across the crate.
//!
//! # Internal Mechanism
//!
//! A typical implementation receives a byte slice, feeds it to the selected
//! hash function, and then wraps the resulting fixed-size digest in a
//! `Hash`. The `Hash` type enforces a constant length via
//! `HASH_LENGTH`; the hasher is responsible
//! for producing exactly that many bytes. If the underlying algorithm
//! produces a digest of a different length, the implementation must either
//! truncate, extend, or return
//! `VctrlError::InvalidHashLength`.
//!
//! # Examples
//!
//! A simple deterministic hasher that returns a constant hash:
//!
//! ```
//! use libvctrl_handler::{Hash, Hasher, VctrlError};
//!
//! struct ConstantHasher;
//!
//! impl Hasher for ConstantHasher {
//!     fn hash(&self, _data: &[u8]) -> Result<Hash, VctrlError> {
//!         Ok(Hash::from_bytes(&[0xAB; 64]).unwrap())
//!     }
//! }
//!
//! let hasher = ConstantHasher;
//! let hash = hasher.hash(b"anything").unwrap();
//! assert_eq!(hash.as_bytes(), &[0xAB; 64]);
//! ```

use crate::errors::VctrlError;
use crate::types::hash::Hash;

/// Defines the interface for hashing raw data into a `Hash`.
///
/// # Purpose
///
/// A `Hasher` implements the specific content-addressing algorithm (e.g.,
/// SHA-256, BLAKE3) used to identify objects in the system. The output is
/// always a `Hash`, which is a fixed-size byte array of
/// `HASH_LENGTH` bytes.
///
/// # Design Rationale
///
/// - The method takes `&self` rather than consuming the hasher, allowing a
///   single instance to be reused for multiple hashing operations.
/// - The method takes `&[u8]` rather than `Vec<u8>` to avoid unnecessary
///   allocation and to accept any byte source (files, network buffers,
///   already-serialized objects).
/// - The return type is `Result<Hash, VctrlError>` to accommodate
///   fallible hashing backends while maintaining a unified error surface.
///
/// # Why Not `Hash::from_bytes` Directly?
///
/// The `Hash` constructor validates length, but it does not compute a
/// digest. The `Hasher` trait is responsible for the actual cryptographic
/// operation. This separation allows the rest of the crate to depend only on
/// the contract, not on a concrete algorithm.
///
/// # How It Works Internally
///
/// An implementation receives raw bytes and returns a `Hash`. It must
/// guarantee that the produced hash has exactly
/// `HASH_LENGTH` bytes. Most implementations
/// will call `Hash::from_bytes` on the digest produced by the underlying
/// hash function. If the digest length is not exactly 64 bytes, the
/// implementation must handle the mismatch, typically by returning
/// `VctrlError::InvalidHashLength`.
///
/// # Examples
///
/// A complete dummy hasher:
///
/// ```
/// use libvctrl_handler::{Hash, Hasher, VctrlError};
///
/// struct DummyHasher;
///
/// impl Hasher for DummyHasher {
///     fn hash(&self, _data: &[u8]) -> Result<Hash, VctrlError> {
///         Ok(Hash::from_bytes(&[0u8; 64]).unwrap())
///     }
/// }
///
/// let hasher = DummyHasher;
/// let hash = hasher.hash(b"hello").unwrap();
/// assert_eq!(hash.as_bytes().len(), 64);
/// ```
pub trait Hasher {
    /// Computes a cryptographic `Hash` from the provided byte slice.
    ///
    /// # Purpose
    ///
    /// This method is the core contract of the `Hasher` trait. It takes an
    /// arbitrary byte slice and returns its content address as a `Hash`.
    /// The same input must always produce the same output for a given
    /// hasher instance.
    ///
    /// # Arguments
    ///
    /// * `data` - The raw bytes to hash. This may be any serialized object,
    ///   file content, or arbitrary payload.
    ///
    /// # Errors
    ///
    /// Returns `VctrlError` if the hashing operation
    /// fails internally. Common failure modes include:
    ///
    /// - `InvalidHashLength`: The underlying algorithm produced a digest of
    ///   an unexpected length.
    /// - `Other`: The hasher is not properly initialized (e.g., missing
    ///   key material for keyed algorithms).
    ///
    /// # How It Works
    ///
    /// The implementation feeds `data` to the underlying hash function,
    /// obtains the digest, and wraps it in a `Hash` via
    /// `Hash::from_bytes`. If the digest length is correct, the call
    /// succeeds; otherwise, an error is returned.
    ///
    /// # Examples
    ///
    /// Hashing a byte slice:
    ///
    /// ```
    /// use libvctrl_handler::{Hash, Hasher, VctrlError};
    ///
    /// struct DummyHasher;
    ///
    /// impl Hasher for DummyHasher {
    ///     fn hash(&self, _data: &[u8]) -> Result<Hash, VctrlError> {
    ///         Ok(Hash::from_bytes(&[0x5A; 64]).unwrap())
    ///     }
    /// }
    ///
    /// let hasher = DummyHasher;
    /// let hash = hasher.hash(b"data").unwrap();
    /// assert_eq!(hash.as_bytes().len(), 64);
    /// assert_eq!(hash.as_bytes()[0], 0x5A);
    /// ```
    ///
    /// Using a hasher to produce a content address for a
    /// `Blob`:
    ///
    /// ```
    /// use libvctrl_handler::{Blob, Hash, Hasher, VctrlError};
    ///
    /// struct DummyHasher;
    ///
    /// impl Hasher for DummyHasher {
    ///     fn hash(&self, _data: &[u8]) -> Result<Hash, VctrlError> {
    ///         Ok(Hash::from_bytes(&[0x11; 64]).unwrap())
    ///     }
    /// }
    ///
    /// let blob = Blob::new(b"hello".to_vec());
    /// let hasher = DummyHasher;
    /// let address = hasher.hash(blob.data()).unwrap();
    /// assert_eq!(address.as_bytes().len(), 64);
    /// ```
    fn hash(&self, data: &[u8]) -> Result<Hash, VctrlError>;
}
