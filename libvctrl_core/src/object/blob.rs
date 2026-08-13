//! Builder pattern for constructing [`Blob`](libvctrl_handler::Blob) objects.
//!
//! # Purpose
//!
//! This module provides the [`BlobBuilder`], an ergonomic utility for
//! incrementally constructing version control blobs. While a [`Blob`] can be
//! created directly via [`Blob::new`](libvctrl_handler::Blob::new), the
//! builder pattern provides a consistent API across all object types in the
//! system.
//!
//! # Design Rationale
//!
//! - **API consistency**: Complex objects like commits and trees have many
//!   fields and benefit greatly from the builder pattern. Providing a builder
//!   for blobs ensures a uniform construction experience across the crate.
//! - **Extensibility**: If future versions of the system require
//!   pre-processing (like compression) or validation (like checking
//!   `MAX_BLOB_SIZE`) before creating a blob, this logic can be added to the
//!   builder without breaking the existing `Blob::new` API.
//! - **Ownership management**: The builder takes ownership of the underlying
//!   `Vec<u8>` during the `with_data` phase. When [`build`] is called, the
//!   vector is moved into the final [`Blob`] with zero heap allocations.
//! - **Fluent interface**: The builder consumes and returns `self` by value,
//!   enabling method chaining that reads naturally.
//!
//! # Internal Mechanism
//!
//! The builder holds a private `Vec<u8>`. The [`build`] method consumes the
//! builder and moves the vector directly into a new [`Blob`] instance.
//! The `new` method initializes an empty buffer; `with_data` replaces the
//! buffer with caller-provided content; and `build` converts the builder into
//! a finalized blob.
//!
//! # Examples
//!
//! Building a blob with data:
//!
//! ```
//! use libvctrl_core::object::BlobBuilder;
//!
//! let blob = BlobBuilder::new()
//!     .with_data(b"file content".to_vec())
//!     .build();
//!
//! assert_eq!(blob.size(), 12);
//! ```
//!
//! Building an empty blob using `Default`:
//!
//! ```
//! use libvctrl_core::object::BlobBuilder;
//!
//! let blob = BlobBuilder::default().build();
//! assert!(blob.is_empty());
//! ```

use libvctrl_handler::Blob;

/// A builder for creating [`Blob`](libvctrl_handler::Blob) objects.
///
/// # Purpose
///
/// Provides a fluent interface for assembling a blob's data before
/// finalizing it into an immutable object.
///
/// # Design Rationale
///
/// Implements the standard builder pattern. It derives [`Default`] so it can
/// be easily instantiated, and [`Debug`] for logging purposes. The `build`
/// method consumes `self`, preventing the reuse of the builder after the
/// data has been moved into the final blob.
///
/// # Field Privacy
///
/// The internal `data` field is private. This encapsulation ensures that
/// all modifications go through the builder's methods, preserving the
/// linear construction flow and preventing direct external mutation.
///
/// # Memory Layout
///
/// The builder owns a [`Vec<u8>`], which consists of a pointer, length, and
/// capacity. When the builder is created, this vector is empty and allocates
/// nothing on the heap. If `with_data` is called, the vector takes ownership
/// of the provided buffer. When `build` is called, the vector is moved into
/// the [`Blob`] without any cloning or heap reallocation.
///
/// # Examples
///
/// Building a blob with some data:
///
/// ```
/// use libvctrl_core::object::BlobBuilder;
///
/// let blob = BlobBuilder::new()
///     .with_data(b"file content".to_vec())
///     .build();
///
/// assert_eq!(blob.size(), 12);
/// ```
///
/// Building an empty blob using `Default`:
///
/// ```
/// use libvctrl_core::object::BlobBuilder;
///
/// let blob = BlobBuilder::default().build();
/// assert!(blob.is_empty());
/// ```
#[derive(Debug, Default)]
pub struct BlobBuilder {
    data: Vec<u8>,
}

impl BlobBuilder {
    /// Creates a new, empty `BlobBuilder`.
    ///
    /// # Design Rationale
    ///
    /// This is a `const fn`, allowing the builder to be instantiated in
    /// compile-time contexts if needed. It initializes the internal buffer
    /// as an empty `Vec`, which does not allocate until data is added.
    ///
    /// # Performance
    ///
    /// Because the vector is created empty, no heap allocation occurs at
    /// construction time. This makes `BlobBuilder::new()` a zero-cost
    /// operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::object::BlobBuilder;
    ///
    /// let builder = BlobBuilder::new();
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Sets the raw data content for the blob.
    ///
    /// # Design Rationale
    ///
    /// This method takes ownership of the provided `Vec<u8>` and returns the
    /// builder by value, enabling method chaining. It replaces any previously
    /// set data. By consuming `self`, the method enforces a linear chain of
    /// configuration calls; you cannot accidentally mutate a builder without
    /// reassigning it.
    ///
    /// # Why take ownership of `Vec<u8>`?
    ///
    /// Taking ownership avoids copying the byte buffer. The caller may have
    /// obtained the data from a file read or network operation; moving it
    /// into the builder transfers the existing allocation without any
    /// additional copying.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::object::BlobBuilder;
    ///
    /// let blob = BlobBuilder::new()
    ///     .with_data(vec![1, 2, 3])
    ///     .build();
    ///
    /// assert_eq!(blob.data(), &[1, 2, 3]);
    /// ```
    #[must_use]
    pub fn with_data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    /// Consumes the builder and returns a finalized
    /// [`Blob`](libvctrl_handler::Blob).
    ///
    /// # Design Rationale
    ///
    /// This method consumes `self` to enforce a linear flow -- the builder
    /// cannot be reused after the data has been extracted. The internal
    /// `Vec<u8>` is moved directly into the `Blob` without cloning, ensuring
    /// zero-cost finalization.
    ///
    /// # Return Value
    ///
    /// A new [`Blob`](libvctrl_handler::Blob) containing the bytes that were
    /// set via [`with_data`](Self::with_data). If no data was set, the
    /// resulting blob is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::object::BlobBuilder;
    /// use libvctrl_handler::Blob;
    ///
    /// let builder = BlobBuilder::new().with_data(b"data".to_vec());
    /// let blob: Blob = builder.build();
    ///
    /// assert_eq!(blob.size(), 4);
    /// ```
    #[must_use]
    pub fn build(self) -> Blob {
        Blob::new(self.data)
    }
}
