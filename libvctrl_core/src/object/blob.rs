//! # Blob Builder
//!
//! This module provides a fluent, ownership-driven builder for constructing
//! [`Blob`] objects. The builder pattern is used because a [`Blob`] is an
//! immutable value object with exactly one required piece of data: the raw
//! content bytes. The builder allows setting that data in a chainable,
//! readable way while deferring validation until the final `build()` call.

use libvctrl_handler::{Blob, VctrlError};

/// A builder for creating [`Blob`] objects.
///
/// `BlobBuilder` provides a safe, ergonomic way to construct a [`Blob`] from a
/// `Vec<u8>` while deferring size validation to the final build step. It is a
/// zero-cost abstraction: after the build, the builder is consumed and the
/// resulting [`Blob`] owns the data with no extra copies.
///
/// # Why this struct exists
///
/// The [`Blob`] constructor `Blob::new` may fail if the supplied data exceeds
/// [`MAX_BLOB_SIZE`](libvctrl_handler::MAX_BLOB_SIZE). A builder delays that
/// fallible operation, allowing callers to accumulate or transform data before
/// finalizing. It also makes construction consistent with other object types
/// that have more fields, providing a uniform API across the crate.
///
/// # How it works
///
/// The builder stores the content in a private `Vec<u8>`. `with_data` replaces
/// that buffer. `build` moves the buffer into `Blob::new`, which performs
/// validation and returns a [`Result`]. After `build`, the builder is consumed
/// and cannot be reused.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// # use libvctrl_core::object::BlobBuilder;
/// let blob = BlobBuilder::new()
///     .with_data(b"file content".to_vec())
///     .build()
///     .unwrap();
///
/// assert_eq!(blob.data(), b"file content");
/// ```
#[derive(Debug, Default)]
pub struct BlobBuilder {
    data: Vec<u8>,
}

impl BlobBuilder {
    /// Creates a new `BlobBuilder` with no data.
    ///
    /// The builder is initially empty. Use [`with_data`](Self::with_data) to
    /// set the content, or call [`build`](Self::build) to produce an empty
    /// [`Blob`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_core::object::BlobBuilder;
    /// let builder = BlobBuilder::new();
    /// let blob = builder.build().unwrap();
    /// assert!(blob.data().is_empty());
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Sets the data for the blob.
    ///
    /// This method consumes `self` and returns a new builder with the given
    /// `data` replacing any previously set content. It does not validate the
    /// size; validation occurs only when [`build`](Self::build) is called.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_core::object::BlobBuilder;
    /// let blob = BlobBuilder::new()
    ///     .with_data(vec![1, 2, 3])
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(blob.data(), &[1, 2, 3]);
    /// ```
    #[must_use]
    pub fn with_data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    /// Builds the [`Blob`].
    ///
    /// This consumes the builder, moves the stored data into the new [`Blob`],
    /// and validates it against the system limits.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the data exceeds
    /// [`MAX_BLOB_SIZE`](libvctrl_handler::MAX_BLOB_SIZE). The exact variant
    /// depends on the implementation in `libvctrl_handler`.
    ///
    /// # Examples
    ///
    /// Successful build:
    ///
    /// ```
    /// # use libvctrl_core::object::BlobBuilder;
    /// let blob = BlobBuilder::new()
    ///     .with_data(b"hello".to_vec())
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(blob.data(), b"hello");
    /// ```
    pub fn build(self) -> Result<Blob, VctrlError> {
        Blob::new(self.data)
    }
}
