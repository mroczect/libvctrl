//! Delta types for representing differences between trees.
//!
//! # Purpose
//!
//! This module defines data structures that describe what changed between
//! two tree objects. These types are used by the `TreeDiffer` trait and
//! by plumbing diff commands such as `diff-tree`, `diff-index`, and
//! `merge-trees`.
//!
//! The module contains:
//!
//! - [`ChangeKind`] – an enum classifying the type of change.
//! - [`FileDelta`] – a single file-level change.
//! - [`TreeDelta`] – a collection of file-level changes representing the
//!   difference between two trees.
//!
//! # Examples
//!
//! Constructing a simple delta between two file versions:
//!
//! ```
//! use std::path::PathBuf;
//! use libvctrl_handler::{ChangeKind, FileDelta, Hash, TreeDelta};
//!
//! let old = Hash::from_bytes(&[0u8; 64]).unwrap();
//! let new = Hash::from_bytes(&[1u8; 64]).unwrap();
//!
//! let file_delta = FileDelta {
//!     path: PathBuf::from("src/main.rs"),
//!     old_hash: Some(old),
//!     new_hash: Some(new),
//!     kind: ChangeKind::Modified,
//! };
//!
//! let tree_delta = TreeDelta {
//!     changes: vec![file_delta],
//! };
//!
//! assert_eq!(tree_delta.changes.len(), 1);
//! assert!(tree_delta.iter().all(|d| d.path.as_os_str() == "src/main.rs"));
//! ```

use std::path::PathBuf;

use crate::Hash;

/// The kind of change detected between two versions of a file.
///
/// This enum mirrors the essential change categories used by diff
/// algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeKind {
    /// The file was added in the new tree.
    Added,
    /// The file was deleted from the old tree.
    Deleted,
    /// The file content or mode was modified.
    Modified,
    /// The file changed type (e.g., blob to tree, or symlink to blob).
    TypeChange,
}

/// Represents a single file-level change between two trees.
///
/// The `old_hash` and `new_hash` fields are optional because a file may be
/// added (`old_hash` is `None`) or deleted (`new_hash` is `None`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileDelta {
    /// The path of the changed file.
    pub path: PathBuf,
    /// The blob hash in the old tree, if the file existed there.
    pub old_hash: Option<Hash>,
    /// The blob hash in the new tree, if the file exists there.
    pub new_hash: Option<Hash>,
    /// The kind of change.
    pub kind: ChangeKind,
}

impl FileDelta {
    /// Creates a new `FileDelta` with the given path and change kind.
    ///
    /// The hash fields are initialized to `None`.
    #[must_use]
    pub const fn new(path: PathBuf, kind: ChangeKind) -> Self {
        Self {
            path,
            old_hash: None,
            new_hash: None,
            kind,
        }
    }

    /// Returns `true` if this delta represents an addition.
    #[must_use]
    pub fn is_added(&self) -> bool {
        self.kind == ChangeKind::Added
    }

    /// Returns `true` if this delta represents a deletion.
    #[must_use]
    pub fn is_deleted(&self) -> bool {
        self.kind == ChangeKind::Deleted
    }

    /// Returns `true` if this delta represents a modification.
    #[must_use]
    pub fn is_modified(&self) -> bool {
        self.kind == ChangeKind::Modified
    }

    /// Returns `true` if this delta represents a type change.
    #[must_use]
    pub fn is_type_change(&self) -> bool {
        self.kind == ChangeKind::TypeChange
    }
}

/// Represents the complete difference between two trees.
///
/// A `TreeDelta` is simply a collection of [`FileDelta`] entries. It is
/// intentionally lightweight and can be constructed manually or returned
/// from a `TreeDiffer` implementation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreeDelta {
    /// The list of file-level changes.
    pub changes: Vec<FileDelta>,
}

impl TreeDelta {
    /// Creates a new empty `TreeDelta`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }

    /// Returns the number of file changes in this delta.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.changes.len()
    }

    /// Returns `true` if there are no changes.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Returns an iterator over the file changes.
    pub fn iter(&self) -> std::slice::Iter<'_, FileDelta> {
        self.changes.iter()
    }
}

impl IntoIterator for TreeDelta {
    type Item = FileDelta;
    type IntoIter = std::vec::IntoIter<FileDelta>;

    fn into_iter(self) -> Self::IntoIter {
        self.changes.into_iter()
    }
}

impl<'a> IntoIterator for &'a TreeDelta {
    type Item = &'a FileDelta;
    type IntoIter = std::slice::Iter<'a, FileDelta>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
