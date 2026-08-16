use crate::errors::VctrlError;

/// A trait for managing a Git index (staging area).
pub trait Index: Send + Sync {
    /// The entry type used by the index.
    type Entry: Send + Sync;

    /// The path type used by the index.
    type Path: Send + Sync;

    /// The tree identifier type.
    type TreeId: Send + Sync;

    /// Adds an entry to the index.
    fn add(&mut self, entry: Self::Entry) -> Result<(), VctrlError>;

    /// Removes an entry from the index by path.
    fn remove(&mut self, path: &Self::Path) -> Result<(), VctrlError>;

    /// Clears all entries from the index.
    fn clear(&mut self) -> Result<(), VctrlError>;

    /// Retrieves an entry by path.
    fn get(&self, path: &Self::Path) -> Result<Option<Self::Entry>, VctrlError>;

    /// Checks if an entry exists by path.
    fn contains(&self, path: &Self::Path) -> Result<bool, VctrlError>;

    /// Returns the number of entries in the index.
    fn len(&self) -> Result<usize, VctrlError>;

    /// Returns `true` if the index is empty.
    fn is_empty(&self) -> Result<bool, VctrlError> {
        Ok(self.len()? == 0)
    }

    /// Returns all entries in the index.
    fn entries(&self) -> Result<Vec<Self::Entry>, VctrlError>;

    /// Writes the current index to a tree object and returns its identifier.
    fn write_tree(&self) -> Result<Self::TreeId, VctrlError>;

    /// Reads a tree into the index.
    fn read_tree(&mut self, tree: &Self::TreeId) -> Result<(), VctrlError>;
}
