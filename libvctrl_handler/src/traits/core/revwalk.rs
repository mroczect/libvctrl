//! Revision walking trait.

use crate::errors::VctrlError;

/// An iterator over commit history.
pub type RevWalkIterator<'a, T> = Box<dyn Iterator<Item = Result<T, VctrlError>> + Send + 'a>;

/// Trait for walking commit history.
pub trait RevWalk: Send + Sync {
    /// The commit identifier type.
    type CommitId: Send + Sync;

    /// Returns an iterator over commit history starting from the given commit.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the walk cannot be initialized.
    fn walk(
        &self,
        start: &Self::CommitId,
    ) -> Result<RevWalkIterator<'_, Self::CommitId>, VctrlError>;
}
