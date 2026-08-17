//! Hashing trait.
//!
//! # Architecture
//! This module defines the abstract contract for computing cryptographic hashes.
//! By abstracting the hashing mechanism into a trait, the crate decouples its
//! content-addressing logic from the specific cryptographic algorithm (e.g., SHA-1,
//! SHA-256, SHA-512). This allows consumers to swap algorithms or inject hardware-accelerated
//! implementations without modifying the core object database logic.
//!
//! # Design Rationale: Streaming Cryptography
//! The trait operates on `R: Read` rather than `&[u8]` or `Vec<u8>`. This is a critical
//! architectural decision for performance and security. Git objects, particularly blobs,
//! can be gigabytes in size. Loading an entire object into memory to hash it would cause
//! severe memory fragmentation and potential out-of-memory (OOM) errors. By requiring a
//! reader, the hasher processes data in fixed-size chunks, maintaining a constant memory
//! footprint regardless of the input size.

use crate::errors::VctrlError;
use crate::types::Hash;
use std::io::Read;

/// Trait for computing hash values.
///
/// # Why this exists
/// In a content-addressable storage (CAS) system, the identifier of an object is derived
/// from its content. This trait provides the contract for that derivation. Separating it
/// from the encoder or storage backend allows for independent optimization and testing
/// of the cryptographic pipeline.
///
/// # How it works
/// The trait uses a generic method (`<R: Read + Send>`) instead of a dynamic trait object
/// (`&mut dyn Read`). This leverages Rust's monomorphization: the compiler generates a
/// specialized version of the `hash` method for every concrete reader type used at runtime.
/// This eliminates dynamic dispatch overhead, allowing the compiler to aggressively inline
/// the read loops and buffering logic.
///
/// # Design Rationale: Thread Safety
/// The trait requires `Send + Sync` on `Self`, and `Send` on the reader `R`. Hashing is
/// a CPU-bound, stateless operation (from the perspective of the hasher). By enforcing
/// thread safety, the engine can safely distribute hashing tasks across a thread pool.
/// For example, when writing a packfile, multiple objects can be hashed concurrently on
/// different threads without requiring external synchronization.
///
/// # Examples
///
/// Implementing the trait for a mock hasher that reads stream to completion:
///
/// ```
/// # use libvctrl_handler::traits::core::hasher::Hasher;
/// # use libvctrl_handler::{Hash, VctrlError};
/// # use std::io::Read;
/// #
/// struct MockHasher;
///
/// impl Hasher for MockHasher {
///     fn hash<R: Read + Send>(&self, mut reader: R) -> Result<Hash, VctrlError> {
///         // In a real implementation, this would update a cryptographic state
///         // (e.g., SHA-512) and finalize it. Here, we just drain the reader.
///         let mut buf = Vec::new();
///         reader.read_to_end(&mut buf)?;
///         // Return a deterministic mock hash
///         Hash::from_bytes(&[0_u8; 64])
///     }
/// }
///
/// let hasher = MockHasher;
/// let data = std::io::Cursor::new(b"some data".to_vec());
/// let hash = hasher.hash(data)?;
/// assert_eq!(hash.as_bytes(), &[0_u8; 64]);
/// # Ok::<(), VctrlError>(())
/// ```
pub trait Hasher: Send + Sync {
    /// Returns the hash of the data read from the given reader.
    ///
    /// # How it works
    /// Reads bytes from the provided reader in chunks until EOF is reached. As data is
    /// read, it is fed into the underlying hashing algorithm's state machine. Once the
    /// stream is exhausted, the final digest is computed and returned as a strongly-typed
    /// [`Hash`]. This ensures that the hash is always the correct length (64 bytes for
    /// SHA-512) as validated by [`Hash::from_bytes`].
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if hashing fails. This typically occurs if the underlying
    /// reader experiences an I/O error (e.g., a broken pipe or disk read failure) during
    /// the streaming process.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::hasher::Hasher;
    /// # use libvctrl_handler::{Hash, VctrlError};
    /// # use std::io::Read;
    /// # struct MockHasher;
    /// # impl Hasher for MockHasher {
    /// #     fn hash<R: Read + Send>(&self, mut reader: R) -> Result<Hash, VctrlError> {
    /// #         let mut buf = Vec::new();
    /// #         reader.read_to_end(&mut buf)?;
    /// #         Hash::from_bytes(&[0_u8; 64])
    /// #     }
    /// # }
    /// let hasher = MockHasher;
    /// let stream = std::io::Cursor::new(b"hash this content".to_vec());
    /// let result = hasher.hash(stream);
    /// assert!(result.is_ok());
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn hash<R: Read + Send>(&self, reader: R) -> Result<Hash, VctrlError>;
}
