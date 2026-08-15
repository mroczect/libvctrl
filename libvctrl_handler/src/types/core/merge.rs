//! Merge-related types.

use std::path::PathBuf;

use crate::Hash;

/// A conflict that occurred during a merge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conflict {
    /// The path with a conflict.
    pub path: PathBuf,
    /// The ancestor blob hash.
    pub ancestor_blob: Hash,
    /// The blob from the current branch.
    pub our_blob: Hash,
    /// The blob from the merging branch.
    pub their_blob: Hash,
}

impl Conflict {
    /// Creates a new conflict.
    #[must_use]
    pub const fn new(path: PathBuf, ancestor_blob: Hash, our_blob: Hash, their_blob: Hash) -> Self {
        Self {
            path,
            ancestor_blob,
            our_blob,
            their_blob,
        }
    }
}

/// The result of a merge operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeResult {
    /// The merge succeeded with the resulting tree hash.
    Success(Hash),
    /// The merge produced conflicts.
    Conflicts(Vec<Conflict>),
}

impl MergeResult {
    /// Returns `true` if the merge succeeded.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    /// Returns `true` if the merge produced conflicts.
    #[must_use]
    pub const fn is_conflicts(&self) -> bool {
        matches!(self, Self::Conflicts(_))
    }

    /// Returns the conflicts if any.
    #[must_use]
    pub fn conflicts(&self) -> Option<&[Conflict]> {
        match self {
            Self::Conflicts(conflicts) => Some(conflicts),
            Self::Success(_) => None,
        }
    }
}
