use crate::errors::VctrlError;
use crate::types::Hash;
use std::io::Read;

/// Trait for transporting Git objects.
pub trait Transport: Send + Sync {
    /// Fetches an object by hash, returning a reader.
    fn fetch_object(&self, hash: &Hash) -> Result<Box<dyn Read + Send + '_>, VctrlError>;

    /// Pushes an object to the remote.
    fn push_object(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError>;
}
