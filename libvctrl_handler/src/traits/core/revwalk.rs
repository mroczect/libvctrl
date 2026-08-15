//! Revision walking trait.

use crate::VctrlError;

/// Trait for walking commit history.
pub trait RevWalk {
    /// The commit identifier type.
    type CommitId;

    /// Returns the parent commit IDs of the given commit.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the commit cannot be read.
    fn parents(&self, id: &Self::CommitId) -> Result<Vec<Self::CommitId>, VctrlError>;
}
