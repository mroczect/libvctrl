//! Pack file reader/writer traits.

use crate::VctrlError;

/// Trait for writing Git pack files.
pub trait PackWriter {
    /// The object identifier type.
    type ObjectId;

    /// Writes an object to the pack.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if writing fails.
    fn write_object(&mut self, id: &Self::ObjectId, data: &[u8]) -> Result<(), VctrlError>;

    /// Finishes writing the pack file.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if finalization fails.
    fn finish(&mut self) -> Result<(), VctrlError>;
}

/// Trait for reading Git pack files.
pub trait PackReader {
    /// The object identifier type.
    type ObjectId;

    /// Reads an object from the pack.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the object cannot be read.
    fn read_object(&self, id: &Self::ObjectId) -> Result<Vec<u8>, VctrlError>;
}
