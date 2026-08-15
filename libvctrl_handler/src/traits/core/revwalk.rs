//! Revision walking trait.

use crate::errors::VctrlError;

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
    ) -> Result<Box<dyn Iterator<Item = Result<Self::CommitId, VctrlError>> + Send + '_>, VctrlError>;
}
