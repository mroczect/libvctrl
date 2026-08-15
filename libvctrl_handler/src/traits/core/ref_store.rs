//! Reference store trait.

use crate::errors::VctrlError;
use crate::types::hash::Hash;

/// A trait for managing Git references (branches, tags, etc.).
pub trait RefStore {
    /// An iterator over reference names.
    type RefsIterator: Iterator<Item = Result<String, VctrlError>>;

    /// Sets a reference to the given hash.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the reference cannot be updated.
    fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError>;

    /// Gets the hash pointed to by a reference.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the reference does not exist or cannot be read.
    fn get_ref(&self, name: &str) -> Result<Hash, VctrlError>;

    /// Deletes a reference.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the deletion fails.
    fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError>;

    /// Lists all reference names.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if listing fails.
    fn list_refs(&self) -> Result<Self::RefsIterator, VctrlError>;
}
