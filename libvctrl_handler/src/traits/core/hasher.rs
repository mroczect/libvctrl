//! Cryptographic hashing to produce content addresses.

use crate::errors::VctrlError;
use crate::types::hash::Hash;

/// Defines the interface for hashing raw data into a [`Hash`].
///
/// # Purpose
///
/// A `Hasher` implements the specific content-addressing algorithm (e.g.,
/// SHA-256, BLAKE3) used to identify objects in the system.
///
/// # Design Rationale
///
/// The `hash` method does not return a `Result` because hashing pure byte
/// slices is an infallible operation. It takes `&self` to allow stateful
/// hashers or those initialized with specific keys.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Blob, Hash, Hasher, VctrlError};
///
/// struct DummyHasher;
/// impl Hasher for DummyHasher {
///     fn hash(&self, _data: &[u8]) -> Result<Hash, VctrlError> {
///         Ok(Hash::from_bytes(&[0u8; 64]).unwrap())
///     }
/// }
///
/// let hasher = DummyHasher;
/// let blob = Blob::new(b"hello".to_vec());
/// let hash = hasher.hash(blob.data()).unwrap();
/// assert_eq!(hash.as_bytes(), &[0u8; 64]);
/// ```
pub trait Hasher {
    /// Computes a cryptographic [`Hash`] from the provided byte slice.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the hashing operation fails internally
    /// (e.g., algorithm constraints, entropy exhaustion for salted hashes).
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Hash, Hasher, VctrlError};
    /// # struct HasherImpl;
    /// # impl Hasher for HasherImpl {
    /// #     fn hash(&self, _d: &[u8]) -> Result<Hash, VctrlError> {
    /// #         Ok(Hash::from_bytes(&[0u8; 64]).unwrap())
    /// #     }
    /// # }
    /// let hasher = HasherImpl;
    /// let hash = hasher.hash(b"data").unwrap();
    /// assert_eq!(hash.as_bytes().len(), 64);
    /// ```
    fn hash(&self, data: &[u8]) -> Result<Hash, VctrlError>;
}
