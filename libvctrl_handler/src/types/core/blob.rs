//! Blob object representation.
//!
//! # Architecture
//! This module defines the [`Blob`] struct, which represents the raw content of
//! a file in the Git object model. Blobs are content-addressable, meaning their
//! identifier is derived directly from their byte content.
//!
//! # Design Rationale: Bounded Allocation
//! Git blobs can range from empty files to massive binaries. Without strict limits,
//! a malicious repository could force the engine to allocate gigabytes of memory,
//! causing denial-of-service (DoS). The [`Blob::new`] constructor enforces
//! [`MAX_BLOB_SIZE`](crate::constants::MAX_BLOB_SIZE), acting as a fail-fast
//! circuit breaker during object construction.

use crate::constants::MAX_BLOB_SIZE;
use crate::errors::VctrlError;

/// A Git blob object (file content).
///
/// # Why this exists
/// Provides a strongly-typed, validated wrapper around raw file bytes. By requiring
/// construction via [`new`](Self::new), the crate guarantees that every `Blob`
/// instance in memory adheres to the crate's size limits. Once constructed, the
/// blob is immutable, ensuring safe, concurrent sharing across threads.
///
/// # How it works
/// The struct takes ownership of a `Vec<u8>`. This is a zero-copy operation from
/// the perspective of the byte buffer itself; the vector's allocation is simply
/// moved into the struct, avoiding expensive memory duplication.
///
/// # Examples
///
/// Creating a valid blob:
///
/// ```
/// # use my_crate::types::core::blob::Blob;
/// # use my_crate::VctrlError;
/// let blob = Blob::new(b"file content".to_vec())?;
/// assert_eq!(blob.size(), 12);
/// # Ok::<(), VctrlError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blob {
    data: Vec<u8>,
}

impl Blob {
    /// Creates a new blob from raw bytes.
    ///
    /// # How it works
    /// Takes ownership of the provided `Vec<u8>`. It checks the vector's length
    /// against [`MAX_BLOB_SIZE`](crate::constants::MAX_BLOB_SIZE). The downcast
    /// from `u64` to `usize` is performed using `try_from` to ensure safe
    /// compilation on 32-bit architectures where `usize` might be smaller than `u64`.
    /// If the limit is exceeded, an error is returned and the original data is dropped.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::ExceededMaxSize`] if the data exceeds `MAX_BLOB_SIZE`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use my_crate::types::core::blob::Blob;
    /// # use my_crate::VctrlError;
    /// let data = b"hello world".to_vec();
    /// let blob = Blob::new(data)?;
    /// assert!(!blob.is_empty());
    /// # Ok::<(), VctrlError>(())
    /// ```
    pub fn new(data: Vec<u8>) -> Result<Self, VctrlError> {
        let max_size = usize::try_from(MAX_BLOB_SIZE).unwrap_or(usize::MAX);
        if data.len() > max_size {
            return Err(VctrlError::ExceededMaxSize(format!(
                "blob size {} exceeds maximum allowed size {}",
                data.len(),
                MAX_BLOB_SIZE
            )));
        }
        Ok(Self { data })
    }

    /// Returns the raw bytes of the blob.
    ///
    /// # How it works
    /// Returns an immutable slice (`&[u8]`) borrowing from the internal vector.
    /// This avoids cloning the data, allowing callers to read the content without
    /// taking ownership.
    ///
    /// # Examples
    ///
    /// ```
    /// # use my_crate::types::core::blob::Blob;
    /// # use my_crate::VctrlError;
    /// let blob = Blob::new(b"raw data".to_vec())?;
    /// assert_eq!(blob.data(), b"raw data");
    /// # Ok::<(), VctrlError>(())
    /// ```
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns the size of the blob in bytes.
    ///
    /// # How it works
    /// Implemented as a `const fn`. This allows the size to be evaluated at compile
    /// time if the blob is constructed from a static context, incurring zero runtime
    /// overhead.
    ///
    /// # Examples
    ///
    /// ```
    /// # use my_crate::types::core::blob::Blob;
    /// # use my_crate::VctrlError;
    /// let blob = Blob::new(b"12345".to_vec())?;
    /// assert_eq!(blob.size(), 5);
    /// # Ok::<(), VctrlError>(())
    /// ```
    #[must_use]
    pub const fn size(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the blob is empty.
    ///
    /// # How it works
    /// Checks if the internal vector has zero length. Like [`size`](Self::size),
    /// this is a `const fn`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use my_crate::types::core::blob::Blob;
    /// # use my_crate::VctrlError;
    /// let blob = Blob::new(Vec::new())?;
    /// assert!(blob.is_empty());
    /// # Ok::<(), VctrlError>(())
    /// ```
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
