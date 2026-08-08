//! Builder for [`Blob`] objects.

use libvctrl_handler::Blob;

/// Builder for [`Blob`] objects.
///
/// Thin wrapper around [`Blob::new`] that allows incremental construction.
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
