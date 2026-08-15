//! Delta and change types.

use std::path::PathBuf;

use crate::Hash;

/// The kind of change between two objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeKind {
    /// The object was added.
    Added,
    /// The object was deleted.
    Deleted,
    /// The object was modified.
    Modified,
    /// The object type changed (e.g., blob to tree).
    TypeChange,
}

/// A single file delta between two trees.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileDelta {
    /// The path of the changed file.
    pub path: PathBuf,
    /// The old hash, if the file previously existed.
    pub old_hash: Option<Hash>,
    /// The new hash, if the file exists now.
    pub new_hash: Option<Hash>,
    /// The kind of change.
    pub kind: ChangeKind,
}

impl FileDelta {
    /// Creates a new `FileDelta` with no hash information.
    #[must_use]
    pub const fn new(path: PathBuf, kind: ChangeKind) -> Self {
        Self {
            path,
            old_hash: None,
            new_hash: None,
            kind,
        }
    }

    /// Returns `true` if this is an addition.
    #[must_use]
    pub fn is_added(&self) -> bool {
        self.kind == ChangeKind::Added
    }

    /// Returns `true` if this is a deletion.
    #[must_use]
    pub fn is_deleted(&self) -> bool {
        self.kind == ChangeKind::Deleted
    }

    /// Returns `true` if this is a modification.
    #[must_use]
    pub fn is_modified(&self) -> bool {
        self.kind == ChangeKind::Modified
    }

    /// Returns `true` if this is a type change.
    #[must_use]
    pub fn is_type_change(&self) -> bool {
        self.kind == ChangeKind::TypeChange
    }
}

/// A collection of file deltas between two trees.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreeDelta {
    /// The list of changes.
    pub changes: Vec<FileDelta>,
}

impl TreeDelta {
    /// Creates an empty `TreeDelta`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }

    /// Returns the number of changes.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.changes.len()
    }

    /// Returns `true` if there are no changes.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Iterates over the changes.
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
