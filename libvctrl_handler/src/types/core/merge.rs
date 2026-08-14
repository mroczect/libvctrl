//! Merge result types for representing three-way merge outcomes.
//!
//! # Purpose
//!
//! This module defines the data structures that describe the result of a
//! three-way merge operation. These types are used by plumbing merge
//! functions and, eventually, by porcelain merge commands.
//!
//! The module contains:
//!
//! - [`MergeResult`] – the overall outcome of a merge.
//! - [`Conflict`] – a single conflicting file within a merge.
//!
//! # Examples
//!
//! Constructing a merge result with one conflict:
//!
//! ```
//! use std::path::PathBuf;
//! use libvctrl_handler::{Conflict, Hash, MergeResult};
//!
//! let ancestor = Hash::from_bytes(&[0u8; 64]).unwrap();
//! let ours = Hash::from_bytes(&[1u8; 64]).unwrap();
//! let theirs = Hash::from_bytes(&[2u8; 64]).unwrap();
//!
//! let conflict = Conflict {
//!     path: PathBuf::from("src/main.rs"),
//!     ancestor_blob: ancestor,
//!     our_blob: ours,
//!     their_blob: theirs,
//! };
//!
//! let result = MergeResult::Conflicts(vec![conflict]);
//! assert!(result.is_conflicts());
//! assert!(!result.is_success());
//! ```

use std::path::PathBuf;

use crate::Hash;

/// Represents a single conflicting file in a three-way merge.
///
/// A conflict occurs when the merge algorithm cannot automatically combine
/// the changes from the ancestor, our side, and their side for a given path.
/// This struct records the file path and the blob hashes of the three
/// versions involved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conflict {
    /// The repository-relative path of the conflicting file.
    pub path: PathBuf,
    /// The blob hash of the file in the common ancestor commit.
    pub ancestor_blob: Hash,
    /// The blob hash of the file in the current branch (ours).
    pub our_blob: Hash,
    /// The blob hash of the file in the branch being merged (theirs).
    pub their_blob: Hash,
}

impl Conflict {
    /// Creates a new conflict with the given path and blob hashes.
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

/// The overall result of a three-way merge.
///
/// A merge either succeeds cleanly and produces a new tree hash, or it
/// encounters one or more conflicts that require manual resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeResult {
    /// The merge was successful and produced a new tree with the given hash.
    Success(Hash),
    /// The merge encountered conflicts that must be resolved manually.
    Conflicts(Vec<Conflict>),
}

impl MergeResult {
    /// Returns `true` if the merge succeeded without conflicts.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    /// Returns `true` if the merge encountered conflicts.
    #[must_use]
    pub const fn is_conflicts(&self) -> bool {
        matches!(self, Self::Conflicts(_))
    }

    /// Returns the list of conflicts, or `None` if the merge succeeded.
    #[must_use]
    pub fn conflicts(&self) -> Option<&[Conflict]> {
        match self {
            Self::Conflicts(conflicts) => Some(conflicts),
            Self::Success(_) => None,
        }
    }
}
