//! Blob object representation.

use crate::constants::MAX_BLOB_SIZE;
use crate::errors::VctrlError;

/// A Git blob object (file content).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blob {
    data: Vec<u8>,
}

impl Blob {
    /// Creates a new blob from raw bytes.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::ExceededMaxSize`] if the data exceeds `MAX_BLOB_SIZE`.
    pub fn new(data: Vec<u8>) -> Result<Self, VctrlError> {
        let max_size = usize::try_from(MAX_BLOB_SIZE).unwrap_or(usize::MAX);
        if data.len() > max_size {
            return Err(VctrlError::ExceededMaxSize(format!(
                "blob size {} exceeds maximum allowed size {}",
                data.len(),
                MAX_BLOB_SIZE
            )));
        }
        Ok(Self { data })
    }

    /// Returns the raw bytes of the blob.
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
