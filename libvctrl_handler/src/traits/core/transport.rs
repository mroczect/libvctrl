//! Transport trait.

use crate::errors::VctrlError;
use crate::types::Hash;
use std::io::Read;

/// Trait for transporting Git objects.
pub trait Transport: Send + Sync {
    /// Fetches an object by hash, returning a reader.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the object cannot be fetched.
    fn fetch_object(&self, hash: &Hash) -> Result<Box<dyn Read + Send + '_>, VctrlError>;

    /// Pushes an object to the remote.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the push fails.
    fn push_object(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError>;
}
