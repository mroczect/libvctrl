use crate::errors::VctrlError;
use crate::types::Hash;

/// A trait for managing Git references (branches, tags, etc.).
pub trait RefStore: Send + Sync {
    /// An iterator over reference names.
    type RefsIterator: Iterator<Item = Result<String, VctrlError>> + Send;

    /// Sets a reference to the given hash.
    fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError>;

    /// Gets the hash pointed to by a reference.
    fn get_ref(&self, name: &str) -> Result<Hash, VctrlError>;

    /// Deletes a reference.
    fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError>;

    /// Lists all reference names.
    fn list_refs(&self) -> Result<Self::RefsIterator, VctrlError>;
}
