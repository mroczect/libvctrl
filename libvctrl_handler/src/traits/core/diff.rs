use crate::errors::VctrlError;
use crate::types::TreeDelta;

/// Trait for computing differences between two trees.
pub trait TreeDiffer: Send + Sync {
    /// The identifier type for a tree.
    type TreeId: Send + Sync;

    /// Computes the list of changes between two trees.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if either tree cannot be loaded or diffing fails.
    fn diff_trees(&self, old: &Self::TreeId, new: &Self::TreeId) -> Result<TreeDelta, VctrlError>;
}
