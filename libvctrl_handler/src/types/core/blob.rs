//! Blob object representation.

/// A Git blob object (file content).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blob {
    data: Vec<u8>,
}

impl Blob {
    /// Creates a new blob from raw bytes.
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Returns the raw bytes of the blob.
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns the size of the blob in bytes.
    #[must_use]
    pub const fn size(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the blob is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
