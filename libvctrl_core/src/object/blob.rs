use libvctrl_handler::{Blob, VctrlError};

/// A builder for creating [`Blob`] objects.
#[derive(Debug, Default)]
pub struct BlobBuilder {
    data: Vec<u8>,
}

impl BlobBuilder {
    /// Creates a new `BlobBuilder`.
    #[must_use]
    pub const fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Sets the data for the blob.
    #[must_use]
    pub fn with_data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    /// Builds the [`Blob`].
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the data exceeds the maximum allowed size.
    pub fn build(self) -> Result<Blob, VctrlError> {
        Blob::new(self.data)
    }
}
