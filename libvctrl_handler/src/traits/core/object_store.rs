use crate::errors::VctrlError;
use crate::types::Hash;
use std::io::Read;

/// A trait for storing and retrieving Git objects.
pub trait ObjectStore: Send + Sync {
    /// Stores an object under the given hash.
    fn put(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError>;

    /// Retrieves an object by hash, returning a reader.
    fn get(&self, hash: &Hash) -> Result<Box<dyn Read + Send + '_>, VctrlError>;

    /// Deletes an object by hash.
    fn delete(&mut self, hash: &Hash) -> Result<(), VctrlError>;

    /// Checks whether an object exists.
    fn exists(&self, hash: &Hash) -> Result<bool, VctrlError>;
}
