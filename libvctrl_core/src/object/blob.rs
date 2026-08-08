//! Builder for [`Blob`] objects.
//!
//! A [`BlobBuilder`] is a thin wrapper around [`Blob::new`] that allows
//! incremental construction. It is the simplest of the builders because
//! a blob has only one field: the raw data.
//!
//! # When to use
//!
//! Use the builder when you want to gather data from multiple sources
//! before finalising the blob. For example, if you are reading a file
//! in chunks and want to store the complete content as a blob, you can
//! collect all chunks into a `Vec<u8>` and then call `.with_data(vec)`.
//!
//! If you already have a `Vec<u8>`, you can just call `Blob::new(data)`
//! directly. The builder is not strictly necessary, but it provides a
//! consistent API across all object types.
//!
//! # Example
//!
//! ```rust
//! use libvctrl_core::object::BlobBuilder;
//! use libvctrl_handler::Blob;
//!
//! let blob = BlobBuilder::new()
//!     .with_data(b"Hello, world!".to_vec())
//!     .build();
//! assert_eq!(blob.data(), b"Hello, world!");
//! ```

use libvctrl_handler::Blob;

/// Builder for [`Blob`] objects.
///
/// Thin wrapper around [`Blob::new`] that allows incremental construction.
///
/// # Example
///
/// ```rust
/// # use libvctrl_core::object::BlobBuilder;
/// let blob = BlobBuilder::new()
///     .with_data(vec![0u8; 10])
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct BlobBuilder {
    data: Vec<u8>,
}

impl BlobBuilder {
    /// Creates a new empty builder.
    #[must_use]
    pub const fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Sets the blob data.
    ///
    /// This replaces any previously set data.
    #[must_use]
    pub fn with_data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    /// Consumes the builder and returns a valid [`Blob`].
    #[must_use]
    pub fn build(self) -> Blob {
        Blob::new(self.data)
    }
}
