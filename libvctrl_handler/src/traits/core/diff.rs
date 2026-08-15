//! Tree differencing trait.

use crate::VctrlError;
use crate::types::ChangeKind;
/// Trait for computing differences between two trees.
pub trait TreeDiffer {
    /// The identifier type for a tree.
    type TreeId;

    /// Computes the list of changes between two trees.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if either tree cannot be loaded or diffing fails.
    fn diff_trees(
        &self,
        old: &Self::TreeId,
        new: &Self::TreeId,
    ) -> Result<Vec<ChangeKind>, VctrlError>;
}
