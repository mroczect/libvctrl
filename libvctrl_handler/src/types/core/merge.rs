//! Merge-related types.
//!
//! # Architecture
//! This module defines the data structures used to represent the outcome of a
//! 3-way merge operation. A 3-way merge uses a common ancestor (the merge base)
//! to reconcile changes between two divergent branches ("ours" and "theirs").
//!
//! # Design Rationale: Hash-Based Conflicts
//! The [`Conflict`] struct stores cryptographic hashes (`ancestor_blob`, `our_blob`,
//! `their_blob`) rather than the raw file contents. This is a critical architectural
//! decision for scalability. Merge orchestration can evaluate thousands of paths.
//! By deferring the loading of actual blob bytes to a specialized merge driver
//! (like `diff3`), the engine can quickly identify conflicts without exhausting
//! memory on large binary files.

use std::path::{Path, PathBuf};

use crate::Hash;

/// A conflict that occurred during a merge.
///
/// # Why this exists
/// Represents a single file path where the "ours" and "theirs" branches made
/// conflicting changes relative to the common ancestor, preventing automatic
/// resolution. This struct provides the necessary references for a UI or a
/// text-merge tool to present the conflict to the user.
///
/// # How it works
/// The struct holds the file path and the [`Hash`] of the blob in each of the
/// three merge stages:
/// - `ancestor_blob`: The state of the file at the merge base.
/// - `our_blob`: The state of the file in the current branch (HEAD).
/// - `their_blob`: The state of the file in the branch being merged in.
///
/// # Examples
///
/// ```
/// # use my_crate::types::core::merge::Conflict;
/// # use my_crate::Hash;
/// # let ancestor = Hash::from_bytes(&[0u8; 64])?;
/// # let ours = Hash::from_bytes(&[1u8; 64])?;
/// # let theirs = Hash::from_bytes(&[2u8; 64])?;
/// let conflict = Conflict::new("src/main.rs".into(), ancestor, ours, theirs);
/// assert_eq!(conflict.path(), std::path::Path::new("src/main.rs"));
/// assert_eq!(conflict.our_blob(), ours);
/// # Ok::<(), my_crate::VctrlError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conflict {
    path: PathBuf,
    ancestor_blob: Hash,
    our_blob: Hash,
    their_blob: Hash,
}

impl Conflict {
    /// Creates a new conflict.
    ///
    /// # How it works
    /// Initializes the conflict record with the path and the three corresponding
    /// blob hashes. This is a `const fn`, allowing the construction of conflict
    /// scenarios at compile time for testing purposes.
    #[must_use]
    pub const fn new(path: PathBuf, ancestor_blob: Hash, our_blob: Hash, their_blob: Hash) -> Self {
        Self {
            path,
            ancestor_blob,
            our_blob,
            their_blob,
        }
    }

    /// Returns the path with a conflict.
    ///
    /// # How it works
    /// Returns a reference to the `PathBuf` where the merge conflict occurred.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the ancestor blob hash.
    ///
    /// # How it works
    /// Returns the `Hash` of the file content from the merge base (the common
    /// ancestor commit).
    #[must_use]
    pub const fn ancestor_blob(&self) -> Hash {
        self.ancestor_blob
    }

    /// Returns the blob from the current branch.
    ///
    /// # How it works
    /// Returns the `Hash` of the file content from the "ours" side of the merge
    /// (typically the current `HEAD`).
    #[must_use]
    pub const fn our_blob(&self) -> Hash {
        self.our_blob
    }

    /// Returns the blob from the merging branch.
    ///
    /// # How it works
    /// Returns the `Hash` of the file content from the "theirs" side of the merge
    /// (the branch being merged into the current one).
    #[must_use]
    pub const fn their_blob(&self) -> Hash {
        self.their_blob
    }
}

/// The result of a merge operation.
///
/// # Why this exists
/// Acts as an Algebraic Data Type (ADT) to represent the binary outcome of a merge.
/// By modeling the result as an enum, the Rust compiler forces the caller to
/// explicitly handle both the success and conflict scenarios at compile time,
/// preventing "forgotten conflict" bugs.
///
/// # How it works
/// - `Success(Hash)`: Indicates a clean merge. Contains the hash of the newly
///   created root tree object.
/// - `Conflicts(Vec<Conflict>)`: Indicates that one or more paths could not be
///   merged automatically. Contains the list of conflicts to be resolved.
///
/// # Examples
///
/// Handling a successful merge:
///
/// ```
/// # use my_crate::types::core::merge::MergeResult;
/// # use my_crate::Hash;
/// # let tree_hash = Hash::from_bytes(&[0u8; 64])?;
/// let result = MergeResult::Success(tree_hash);
/// assert!(result.is_success());
/// assert!(result.conflicts().is_none());
/// # Ok::<(), my_crate::VctrlError>(())
/// ```
///
/// Handling a conflicted merge:
///
/// ```
/// # use my_crate::types::core::merge::{Conflict, MergeResult};
/// # use my_crate::Hash;
/// # let h = Hash::from_bytes(&[1u8; 64])?;
/// let result = MergeResult::Conflicts(vec![Conflict::new("file.txt".into(), h, h, h)]);
/// assert!(result.is_conflicts());
/// assert_eq!(result.conflicts().unwrap().len(), 1);
/// # Ok::<(), my_crate::VctrlError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeResult {
    /// The merge succeeded with the resulting tree hash.
    Success(Hash),
    /// The merge produced conflicts.
    Conflicts(Vec<Conflict>),
}

impl MergeResult {
    /// Returns `true` if the merge succeeded.
    ///
    /// # How it works
    /// Uses pattern matching to check if the result is the `Success` variant.
    /// This is a `const fn`, incurring zero runtime overhead.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    /// Returns `true` if the merge produced conflicts.
    ///
    /// # How it works
    /// Uses pattern matching to check if the result is the `Conflicts` variant.
    /// This is a `const fn`, incurring zero runtime overhead.
    #[must_use]
    pub const fn is_conflicts(&self) -> bool {
        matches!(self, Self::Conflicts(_))
    }

    /// Returns the conflicts if any.
    ///
    /// # How it works
    /// If the result is `Conflicts`, it returns `Some(&[Conflict])` borrowing from
    /// the internal vector. If the result is `Success`, it returns `None`. This
    /// avoids cloning the conflict data if the caller only needs to inspect it.
    #[must_use]
    pub fn conflicts(&self) -> Option<&[Conflict]> {
        match self {
            Self::Conflicts(conflicts) => Some(conflicts),
            Self::Success(_) => None,
        }
    }
}
