//! A binary large object (blob) storing raw byte content.
//!
//! `Blob` is the simplest object in the version‑control system. It holds an
//! immutable sequence of bytes exactly as supplied, without interpretation.
//! The internal storage is a [`Vec<u8>`], which gives full ownership of the
//! data to the `Blob` and avoids lifetime annotations for the struct itself.
//!
//! # Why `Vec<u8>`?
//!
//! - **Ownership**: The blob owns its data, so it can be freely moved, cloned,
//!   or stored without borrowing concerns.
//! - **Flexibility**: Callers can provide content from any source – file reads,
//!   network buffers, or in‑memory data – by passing a `Vec<u8>`.
//! - **Efficiency**: Access via [`data()`](Self::data) returns a `&[u8]` with no
//!   allocation, and methods like [`size()`](Self::size) are `const` for
//!   compile‑time evaluation.
//!
//! # Immutability
//!
//! Once constructed, the blob’s content cannot be modified (the `data` field
//! is private and no mutable access is provided). This reflects the
//! content‑addressable nature of the system: a blob is identified by the
//! hash of its data, so changing the data would change its identity.
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
    /// content. No validation or transformation is performed – the bytes are
    /// stored exactly as given.
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
    /// The returned slice lives as long as the `Blob` and provides read‑only
    /// access. There is no copying – the slice points directly into the
    /// struct’s internal buffer.
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
