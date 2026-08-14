//! A binary large object (blob) storing raw byte content.
//!
//! # Purpose
//!
//! `Blob` is the simplest object in the version-control system. It holds an
//! immutable sequence of bytes exactly as supplied, without interpretation.
//! A blob represents the content of a file stored in a version control
//! repository. It is the leaf node of the object graph: it is referenced by
//! tree entries and identified by its content hash.
//!
//! # Design Rationale
//!
//! The blob is deliberately minimal. It does not parse, validate, or
//! transform its contents. This design keeps the object model pure and
//! ensures that any byte sequence can be stored, including binary data,
//! executable files, or symlink targets.
//!
//! ## Why `Vec<u8>`?
//!
//! - **Ownership**: The blob owns its data, so it can be freely moved,
//!   cloned, or stored without borrowing concerns. There are no lifetime
//!   parameters on the struct.
//! - **Flexibility**: Callers can provide content from any source - file
//!   reads, network buffers, or in-memory data - by passing a `Vec<u8>`.
//! - **Efficiency**: Access via [`data()`](Blob::data) returns a `&[u8]`
//!   with no allocation, and methods like [`size()`](Blob::size) are
//!   `const` for compile-time evaluation.
//!
//! ## Immutability
//!
//! Once constructed, the blob's content cannot be modified. The `data`
//! field is private and no mutable accessor is provided. This reflects the
//! content-addressable nature of the system: a blob is identified by the
//! hash of its data, so changing the data would change its identity.
//!
//! # Memory Layout
//!
//! A `Blob` is a single owning pointer to a heap-allocated buffer. Its size
//! is exactly the size of a `Vec<u8>` (24 bytes on 64-bit platforms):
//! a pointer, a length, and a capacity. The actual byte content lives on
//! the heap. Cloning a blob performs a deep copy of the underlying buffer.
//!
//! # Thread Safety
//!
//! `Blob` is `Send` and `Sync` because `Vec<u8>` is both. This allows blobs
//! to be shared across threads without additional synchronization.
//!
//! # Relationship to Other Types
//!
//! - A [`TreeEntry`] references a blob by its
//!   `Hash`(crate::Hash) and an [`EntryKind`] of
//!   `Blob`, `Executable`, or `Symlink`.
//! - The [`Hasher`] trait computes a blob's hash from
//!   [`data()`](Blob::data).
//! - The [`Encoder`] trait serializes a blob into bytes for
//!   storage.
//!
//! # Examples
//!
//! Basic construction and access:
//!
//! ```
//! use libvctrl_handler::types::core::Blob;
//!
//! let content = b"hello, world".to_vec();
//! let blob = Blob::new(content);
//!
//! assert_eq!(blob.data(), b"hello, world");
//! assert_eq!(blob.size(), 12);
//! assert!(!blob.is_empty());
//! ```
//!
//! Empty blob:
//!
//! ```
//! use libvctrl_handler::types::core::Blob;
//!
//! let empty = Blob::new(Vec::new());
//! assert!(empty.is_empty());
//! assert_eq!(empty.size(), 0);
//! ```

/// A binary large object storing raw byte content.
///
/// # Overview
///
/// `Blob` is a wrapper around a private [`Vec<u8>`] that provides read-only
/// access to the bytes. It is the fundamental unit of file content in the
/// version control system.
///
/// # Design Rationale
///
/// - The `data` field is private to enforce immutability after construction.
/// - The struct derives [`Clone`], [`Debug`], [`PartialEq`], and [`Eq`],
///   making it easy to duplicate, print, and compare blobs.
/// - No `Default` implementation is provided because an empty blob should be
///   created explicitly, making the intent clear.
///
/// # Examples
///
/// Basic construction and access:
///
/// ```
/// use libvctrl_handler::types::core::Blob;
///
/// let content = b"hello, world".to_vec();
/// let blob = Blob::new(content);
///
/// assert_eq!(blob.data(), b"hello, world");
/// assert_eq!(blob.size(), 12);
/// assert!(!blob.is_empty());
/// ```
///
/// Empty blob:
///
/// ```
/// use libvctrl_handler::types::core::Blob;
///
/// let empty = Blob::new(Vec::new());
/// assert!(empty.is_empty());
/// assert_eq!(empty.size(), 0);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blob {
    data: Vec<u8>,
}

impl Blob {
    /// Creates a new `Blob` from an owned byte vector.
    ///
    /// The provided `data` is moved into the blob and becomes its entire
    /// content. No validation or transformation is performed - the bytes are
    /// stored exactly as given.
    ///
    /// # Parameters
    ///
    /// * `data` - The byte content to store. Ownership is transferred to the
    ///   blob.
    ///
    /// # Returns
    ///
    /// A new `Blob` instance wrapping the provided bytes.
    ///
    /// # Why not `Default`?
    ///
    /// This constructor requires the caller to explicitly provide content.
    /// An empty blob can still be created by passing `Vec::new()`, but the
    /// intent is clearer.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::Blob;
    ///
    /// let blob = Blob::new(b"sample".to_vec());
    /// assert_eq!(blob.data(), b"sample");
    /// ```
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Returns a reference to the raw byte content.
    ///
    /// The returned slice lives as long as the `Blob` and provides read-only
    /// access. There is no copying - the slice points directly into the
    /// struct's internal buffer.
    ///
    /// # Returns
    ///
    /// A byte slice (`&[u8]`) representing the entire blob content.
    ///
    /// # Why a slice and not a `Vec`?
    ///
    /// Returning a slice avoids exposing the internal capacity and prevents
    /// callers from mutating the content. It also allows zero-copy access to
    /// the data.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::Blob;
    ///
    /// let blob = Blob::new(vec![0, 1, 2]);
    /// assert_eq!(blob.data()[1], 1);
    /// ```
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns the number of bytes in the blob.
    ///
    /// This is a `const fn` so it can be evaluated at compile time when
    /// the blob instance is available in a constant context.
    ///
    /// # Returns
    ///
    /// The length of the blob content in bytes.
    ///
    /// # Performance
    ///
    /// The method simply delegates to [`Vec::len`], which is O(1) and does
    /// not traverse the data.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::Blob;
    ///
    /// let blob = Blob::new(b"12345".to_vec());
    /// assert_eq!(blob.size(), 5);
    /// ```
    #[must_use]
    pub const fn size(&self) -> usize {
        self.data.len()
    }

    /// Checks whether the blob contains zero bytes.
    ///
    /// # Returns
    ///
    /// `true` if the blob content is empty, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::Blob;
    ///
    /// let empty = Blob::new(Vec::new());
    /// assert!(empty.is_empty());
    ///
    /// let non_empty = Blob::new(b"x".to_vec());
    /// assert!(!non_empty.is_empty());
    /// ```
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
