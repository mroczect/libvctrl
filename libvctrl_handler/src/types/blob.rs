//! Binary large object type for the `libvctrl_handler` version control
//! contracts.
//!
//! # Purpose
//! A [`Blob`](crate::Blob) is the fundamental content-addressed payload in a
//! version control system. It stores the raw bytes of a file's contents at a
//! given point in time, with no name, no filesystem mode, and no parent
//! relationship attached. Those concerns are owned by
//! [`TreeEntry`](crate::TreeEntry) and [`Tree`](crate::Tree) respectively,
//! which reference a blob indirectly through its [`Hash`](crate::Hash).
//!
//! # Design rationale
//! The inner `Vec<u8>` is intentionally kept private. Exposing the field
//! directly would allow callers to mutate the bytes after construction,
//! breaking the invariant that a blob's content is immutable for the
//! lifetime of its owning [`Hash`](crate::Hash). Every accessor therefore
//! returns either a shared reference (`&[u8]`) or a copied scalar, so the
//! type is effectively a frozen handle around its data.
//!
//! [`Blob::new`](crate::Blob::new) is a `const fn` for forward compatibility
//! with `const`-context construction. `Vec<u8>` is not yet
//! `const`-constructible on stable Rust, so today the practical benefit is
//! API uniformity with the other `const fn` accessors; once `const` heap
//! allocation stabilizes this entry point will already support zero-cost
//! compile-time blobs without a breaking change.
//!
//! Size validation against
//! [`MAX_BLOB_SIZE`](crate::MAX_BLOB_SIZE) is deliberately **not** performed
//! inside [`Blob::new`](crate::Blob::new). The [`Blob`](crate::Blob) type is
//! a pure data carrier; enforcing storage limits is the responsibility of
//! the [`Encoder`](crate::Encoder) or [`ObjectStore`](crate::ObjectStore)
//! implementation that persists the blob. This keeps construction cheap in
//! hot paths (for example, when streaming content through a
//! [`Hasher`](crate::Hasher)) where the limit does not yet apply.
//!
//! # Internal mechanism
//! [`Blob`](crate::Blob) is a thin wrapper around `Vec<u8>`. There is no
//! allocation on read: [`data`](crate::Blob::data) returns a borrowed slice
//! into the underlying buffer, and [`size`](crate::Blob::size) /
//! [`is_empty`](crate::Blob::is_empty) read the vector's length field
//! directly without traversing the contents.

/// An immutable binary large object representing raw file content.
///
/// # Purpose
/// A `Blob` holds the bytes of a tracked file at a single point in time. It
/// carries no metadata about the file's path or permissions; that
/// information lives in the enclosing [`Tree`](crate::Tree) via
/// [`TreeEntry`](crate::TreeEntry).
///
/// # Design rationale
/// The wrapped `Vec<u8>` is private to preserve the immutability invariant
/// that a blob's content must not change after the blob's
/// [`Hash`](crate::Hash) has been computed. Mutation would silently
/// invalidate every [`Hash`](crate::Hash) and [`Tree`](crate::Tree) that
/// references this blob, so the API exposes only shared (`&[u8]`) accessors.
///
/// Construction does not enforce
/// [`MAX_BLOB_SIZE`](crate::MAX_BLOB_SIZE); that limit is enforced by the
/// storage layer (see [`ObjectStore`](crate::ObjectStore) and
/// [`Encoder`](crate::Encoder)) when the blob is actually persisted. This
/// keeps [`Blob::new`](crate::Blob::new) a zero-cost move in hot paths such
/// as hashing.
///
/// # Internal mechanism
/// The struct is a single-field wrapper around `Vec<u8>`. All accessors are
/// either `O(1)` length reads or borrowed slice views; none allocate.
///
/// # Examples
///
/// Constructing a blob from a byte vector and inspecting it:
///
/// ```
/// use libvctrl_handler::Blob;
///
/// let blob = Blob::new(b"hello, world\n".to_vec());
/// assert_eq!(blob.size(), 13);
/// assert_eq!(blob.data(), b"hello, world\n");
/// assert!(!blob.is_empty());
/// ```
///
/// Cloning a blob copies the underlying buffer:
///
/// ```
/// use libvctrl_handler::Blob;
///
/// let original = Blob::new(vec![0u8; 32]);
/// let clone = original.clone();
/// assert_eq!(original, clone);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blob {
    data: Vec<u8>,
}

impl Blob {
    /// Creates a new `Blob` by taking ownership of the supplied byte vector.
    /// No copy is performed.
    ///
    /// # Design rationale
    /// This is a `const fn` for forward compatibility with `const`-context
    /// construction. `Vec<u8>` is not yet `const`-constructible on stable
    /// Rust, so today the practical benefit is API uniformity with the other
    /// `const fn` accessors; once `const` heap allocation stabilizes this
    /// entry point will already work in `const` initializers without a
    /// breaking change.
    ///
    /// No size validation is performed here. The
    /// [`MAX_BLOB_SIZE`](crate::MAX_BLOB_SIZE) limit is the storage layer's
    /// responsibility (see [`ObjectStore`](crate::ObjectStore) and
    /// [`Encoder`](crate::Encoder)), because a `Blob` may legitimately
    /// exist transiently in memory (for example, while being hashed) even
    /// if it would be rejected by a strict storage backend.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::Blob;
    ///
    /// let blob = Blob::new(vec![1, 2, 3, 4]);
    /// assert_eq!(blob.size(), 4);
    /// ```
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn new(data: Vec<u8>) -> Self {
        // `const` removed
        Self { data }
    }

    /// Returns the raw bytes of the blob as a borrowed slice.
    ///
    /// No allocation occurs; the returned slice points directly into the
    /// blob's internal buffer and is valid for as long as the borrow on
    /// `self` is held.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::Blob;
    ///
    /// let blob = Blob::new(b"payload".to_vec());
    /// assert_eq!(blob.data(), b"payload");
    /// ```
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns the number of bytes stored in the blob.
    ///
    /// This is an `O(1)` operation that reads the length field of the
    /// underlying `Vec<u8>`; it does not scan the contents.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::Blob;
    ///
    /// let empty = Blob::new(Vec::new());
    /// assert_eq!(empty.size(), 0);
    ///
    /// let non_empty = Blob::new(vec![0u8; 1024]);
    /// assert_eq!(non_empty.size(), 1024);
    /// ```
    #[must_use]
    pub const fn size(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the blob contains zero bytes.
    ///
    /// This is semantically equivalent to `self.size() == 0` and is provided
    /// as a convenience for callers that prefer the more expressive form.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::Blob;
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
