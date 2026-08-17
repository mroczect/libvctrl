//! SHA-512 hasher implementation for content addressing.
//!
//! # Why this module exists
//!
//! The [`libvctrl_handler`] crate defines the [`Hasher`](libvctrl_handler::Hasher)
//! trait as the abstraction for content-addressable object hashing. This module
//! provides a concrete implementation using the SHA-512 algorithm from the
//! [`libvctrl_sha512`] crate. It bridges the raw SHA-512 digest computation to
//! the handler's [`Hash`] type, ensuring that all hashes produced by this
//! crate are compatible with the rest of the VCS ecosystem.
//!
//! # How it works
//!
//! The [`Sha512Hasher`] is a zero-sized struct. It holds no state because
//! hashing is stateless across invocations. The [`hash`](Sha512Hasher::hash)
//! method reads from a generic [`Read`](std::io::Read) stream in fixed-size
//! chunks, feeds each chunk into the underlying [`Sha512Hash`] engine, and
//! finalizes the digest into a 64-byte [`Hash`]. The result length always
//! matches [`HASH_LENGTH`](libvctrl_handler::HASH_LENGTH), so conversion
//! cannot fail.
//!
//! # Examples
//!
//! Hash a byte slice:
//!
//! ```
//! use libvctrl_core::hash::Sha512Hasher;
//! use libvctrl_handler::Hasher;
//!
//! let hasher = Sha512Hasher;
//! let hash = hasher.hash(b"hello world".as_ref()).unwrap();
//! assert_eq!(hash.as_bytes().len(), 64);
//! ```

use libvctrl_handler::{Hash, Hasher, VctrlError};
use libvctrl_sha512::Hash as Sha512Hash;

/// A hasher that uses the SHA-512 algorithm.
///
/// # Design rationale
///
/// This is a zero-sized struct (ZST) because the SHA-512 algorithm does not
/// require any persistent state between calls. Each call to
/// [`hash`](Sha512Hasher::hash) creates a fresh [`Sha512Hash`] engine,
/// processes the input, and drops it. This makes the hasher trivially
/// [`Clone`], [`Default`], and [`Debug`], and allows it to be passed by value
/// without overhead.
///
/// The struct name follows the convention of naming the concrete implementation
/// after the algorithm it uses, making it obvious to users what cryptographic
/// function will be applied.
///
/// # Examples
///
/// Create a hasher instance:
///
/// ```
/// # use libvctrl_core::hash::Sha512Hasher;
/// let hasher = Sha512Hasher::default();
/// // The hasher is stateless and can be reused for multiple inputs.
/// ```
#[derive(Debug, Default, Clone)]
pub struct Sha512Hasher;

impl Hasher for Sha512Hasher {
    /// Hashes the contents of a reader using SHA-512.
    ///
    /// # How it works
    ///
    /// The method reads from `reader` in 4096-byte chunks to avoid loading
    /// large objects entirely into memory. For each chunk, it calls
    /// [`update`](Sha512Hash::update) on a fresh [`Sha512Hash`] engine. Once
    /// EOF is reached (read returns 0), the engine is finalized and the raw
    /// 64-byte digest is converted into a [`Hash`] via
    /// [`Hash::from_bytes`]. Because SHA-512 always produces 64 bytes, the
    /// conversion cannot fail and the `?` operator is safe to use.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::IoError`] if an I/O error occurs while reading
    /// from the underlying reader. Hash computation itself is infallible.
    ///
    /// # Examples
    ///
    /// Hash data from a [`Cursor`](std::io::Cursor):
    ///
    /// ```
    /// # use libvctrl_core::hash::Sha512Hasher;
    /// # use libvctrl_handler::Hasher;
    /// # use std::io::Cursor;
    /// let hasher = Sha512Hasher;
    /// let data = b"streaming data";
    /// let hash = hasher.hash(Cursor::new(data)).unwrap();
    /// assert_eq!(hash.as_bytes().len(), 64);
    /// ```
    fn hash<R: std::io::Read + Send>(&self, mut reader: R) -> Result<Hash, VctrlError> {
        let mut hasher = Sha512Hash::new();
        let mut buffer = [0u8; 4096];
        loop {
            let n = reader.read(&mut buffer).map_err(VctrlError::from_io)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        let digest = hasher.finalize();
        Hash::from_bytes(&digest)
    }
}
