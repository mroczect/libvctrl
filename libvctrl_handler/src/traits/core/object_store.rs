//! Object store trait.

use crate::errors::VctrlError;
use crate::types::Hash;
use std::io::Read;

/// A trait for storing and retrieving Git objects.
pub trait ObjectStore: Send + Sync {
    /// Stores an object under the given hash.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the object cannot be stored.
    fn put(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError>;

    /// Retrieves an object by hash, returning a reader.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the object cannot be found or read.
    fn get(&self, hash: &Hash) -> Result<Box<dyn Read + Send + '_>, VctrlError>;

    /// Deletes an object by hash.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the deletion fails.
    fn delete(&mut self, hash: &Hash) -> Result<(), VctrlError>;

    /// Checks whether an object exists.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the existence check fails.
    fn exists(&self, hash: &Hash) -> Result<bool, VctrlError>;
}
