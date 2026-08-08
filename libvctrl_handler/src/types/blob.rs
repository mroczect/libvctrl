//! # Blob – Raw File Content
//!
//! A `Blob` represents the content of a file as a sequence of raw bytes.
//! It is the simplest object type in `libvctrl` and serves as the leaf node
//! in the tree structure (directories point to blobs via hash).
//!
//! ## Content‑Addressability
//!
//! Although a `Blob` does not store its own hash, the hash is computed
//! **externally** from its bytes (using a [`Hasher`](crate::Hasher)).
//! This hash becomes the object’s identifier and is stored in tree entries
//! and commits. This design ensures that:
//!
//! - **Integrity** – any change to the bytes changes the hash, making corruption
//!   easily detectable.
//! - **Deduplication** – identical content produces the same hash, so it is
//!   stored only once.
//!
//! ## Size Considerations
//!
//! `Blob` itself imposes **no** size limit at the type level – it can hold any
//! `Vec<u8>` that fits in memory. However, **decoders** that parse untrusted
//! input **must** enforce [`MAX_BLOB_SIZE`](crate::MAX_BLOB_SIZE) (100 MiB)
//! to prevent denial‑of‑service attacks. The reference implementation in
//! `libvctrl_core` does this automatically.
//!
//! ## Empty Blobs
//!
//! An empty blob (`Blob::new(vec![])`) is fully supported and represents an
//! empty file. Its hash is the hash of the empty byte sequence.
//!
//! # Examples
//!
//! ## Creating a Blob and Computing Its Hash
//!
//! ```rust
//! use libvctrl_handler::{Blob, Hash, Hasher, HASH_LENGTH};
//! # // dummy hasher for demonstration
//! # struct DummyHasher;
//! # impl Hasher for DummyHasher {
//! #     fn hash(&self, data: &[u8]) -> Hash {
//! #         // In production, use SHA-512.
//! #         Hash::from_bytes(&[0xAA; HASH_LENGTH]).unwrap()
//! #     }
//! # }
//!
//! // 1. Create a blob with some data
//! let data = b"Hello, world!".to_vec();
//! let blob = Blob::new(data);
//!
//! // 2. Compute its hash using a hasher (e.g., SHA-512)
//! let hasher = DummyHasher;
//! let hash = hasher.hash(blob.data());
//!
//! // 3. Store the blob and its hash in an ObjectStore (not shown)
//! // The hash can now be used in tree entries and commits.
//! ```
//!
//! ## Working with Blob Methods
//!
//! ```rust
//! use libvctrl_handler::Blob;
//!
//! let blob = Blob::new(b"Hello".to_vec());
//! assert_eq!(blob.size(), 5);
//! assert!(!blob.is_empty());
//!
//! let empty = Blob::new(vec![]);
//! assert!(empty.is_empty());
//! assert_eq!(empty.size(), 0);
//! ```
//!
//! ## Serialization and Deserialization
//!
//! A `Blob` is typically encoded to bytes (via an [`Encoder`](crate::Encoder))
//! before being written to storage. The reference binary format simply prefixes
//! the data with its length, making decoding straightforward. The decoder
//! enforces the `MAX_BLOB_SIZE` limit.
//!
//! # Relation to Other Types
//!
//! - [`TreeEntry`](crate::TreeEntry) – uses `EntryKind::Blob` to point to a blob.
//! - [`Tree`](crate::Tree) – contains multiple entries, each referencing a blob or
//!   another tree.
//! - [`Commit`](crate::Commit) – references a root tree, which eventually references
//!   blobs.
//! - [`Hash`](crate::Hash) – the identifier derived from the blob’s bytes.

/// A blob object – raw, uninterpreted data.
///
/// Represents the contents of a file. No encoding, compression, or metadata
/// is stored – just the raw bytes.
///
/// # Empty blobs
/// An empty blob (`Blob::new(vec![])`) is perfectly valid and represents
/// an empty file.
///
/// # Size limits
/// There is **no** size limit enforced at the type level. A `Blob` can hold
/// any amount of data that fits in memory. However, decoders that process
/// untrusted input **should** respect [`MAX_BLOB_SIZE`](crate::constants::MAX_BLOB_SIZE)
/// to prevent memory‑exhaustion attacks. The reference decoder in
/// `libvctrl_core` enforces this limit.
///
/// # Example
///
/// ```rust
/// use libvctrl_handler::Blob;
///
/// let data = b"Hello, world!".to_vec();
/// let blob = Blob::new(data.clone());
/// assert_eq!(blob.data(), b"Hello, world!");
/// assert_eq!(blob.size(), 13);
/// assert!(!blob.is_empty());
///
/// // Empty blob
/// let empty = Blob::new(vec![]);
/// assert!(empty.is_empty());
/// assert_eq!(empty.size(), 0);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blob {
    data: Vec<u8>,
}

impl Blob {
    /// Creates a new `Blob` with the given data.
    ///
    /// This constructor does **not** validate the size, as the size limit is
    /// only a recommendation for decoders. It is the caller’s responsibility
    /// to ensure that the data is not excessively large if memory constraints
    /// are a concern.
    #[must_use]
    pub const fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Returns a reference to the blob's raw data.
    ///
    /// This gives read‑only access to the internal byte vector. Use this
    /// to pass the data to a hasher, encoder, or other processing functions.
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns the size of the blob in bytes.
    ///
    /// Equivalent to `self.data().len()`.
    #[must_use]
    pub const fn size(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the blob contains no data.
    ///
    /// This is a convenient shorthand for `self.size() == 0`.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
