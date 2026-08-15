//! Transport trait.

use crate::errors::VctrlError;
use crate::types::hash::Hash;

/// Trait for transporting Git objects.
pub trait Transport {
    /// Fetches an object by hash.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the object cannot be fetched.
    fn fetch_object(&self, hash: &Hash) -> Result<Vec<u8>, VctrlError>;

    /// Pushes an object to the remote.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the push fails.
    fn push_object(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError>;
}
