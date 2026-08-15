//! Pack file reader/writer traits.

use crate::errors::VctrlError;
use std::io::Read;

/// Trait for writing Git pack files.
pub trait PackWriter: Send + Sync {
    /// The object identifier type.
    type ObjectId: Send + Sync;

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
pub trait PackReader: Send + Sync {
    /// The object identifier type.
    type ObjectId: Send + Sync;

    /// Reads an object from the pack, returning a reader.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the object cannot be read.
    fn read_object(&self, id: &Self::ObjectId) -> Result<Box<dyn Read + Send + '_>, VctrlError>;
}
