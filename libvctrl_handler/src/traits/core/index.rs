//! Index trait.

use crate::VctrlError;

/// A trait for managing a Git index (staging area).
pub trait Index {
    /// The entry type used by the index.
    type Entry;

    /// The path type used by the index.
    type Path;

    /// The tree identifier type.
    type TreeId;

    /// Adds an entry to the index.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the operation fails.
    fn add(&mut self, entry: Self::Entry) -> Result<(), VctrlError>;

    /// Removes an entry from the index by path.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the operation fails.
    fn remove(&mut self, path: &Self::Path) -> Result<(), VctrlError>;

    /// Clears all entries from the index.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the operation fails.
    fn clear(&mut self) -> Result<(), VctrlError>;

    /// Writes the current index to a tree object and returns its identifier.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the tree cannot be written.
    fn write_tree(&self) -> Result<Self::TreeId, VctrlError>;

    /// Reads a tree into the index.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the tree cannot be read.
    fn read_tree(&mut self, tree: &Self::TreeId) -> Result<(), VctrlError>;
}
