use std::path::{Path, PathBuf};

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
    /// The object was renamed.
    Renamed,
    /// The object was copied.
    Copied,
}

/// A single file delta between two trees.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileDelta {
    path: PathBuf,
    old_path: Option<PathBuf>,
    old_hash: Option<Hash>,
    new_hash: Option<Hash>,
    kind: ChangeKind,
}

impl FileDelta {
    /// Creates a new `FileDelta` representing an addition.
    #[must_use]
    pub const fn added(path: PathBuf, new_hash: Hash) -> Self {
        Self {
            path,
            old_path: None,
            old_hash: None,
            new_hash: Some(new_hash),
            kind: ChangeKind::Added,
        }
    }

    /// Creates a new `FileDelta` representing a deletion.
    #[must_use]
    pub const fn deleted(path: PathBuf, old_hash: Hash) -> Self {
        Self {
            path,
            old_path: None,
            old_hash: Some(old_hash),
            new_hash: None,
            kind: ChangeKind::Deleted,
        }
    }

    /// Creates a new `FileDelta` representing a modification.
    #[must_use]
    pub const fn modified(path: PathBuf, old_hash: Hash, new_hash: Hash) -> Self {
        Self {
            path,
            old_path: None,
            old_hash: Some(old_hash),
            new_hash: Some(new_hash),
            kind: ChangeKind::Modified,
        }
    }

    /// Creates a new `FileDelta` representing a type change.
    #[must_use]
    pub const fn type_change(path: PathBuf, old_hash: Hash, new_hash: Hash) -> Self {
        Self {
            path,
            old_path: None,
            old_hash: Some(old_hash),
            new_hash: Some(new_hash),
            kind: ChangeKind::TypeChange,
        }
    }

    /// Creates a new `FileDelta` representing a rename.
    #[must_use]
    pub const fn renamed(
        old_path: PathBuf,
        new_path: PathBuf,
        old_hash: Hash,
        new_hash: Hash,
    ) -> Self {
        Self {
            path: new_path,
            old_path: Some(old_path),
            old_hash: Some(old_hash),
            new_hash: Some(new_hash),
            kind: ChangeKind::Renamed,
        }
    }

    /// Creates a new `FileDelta` representing a copy.
    #[must_use]
    pub const fn copied(
        old_path: PathBuf,
        new_path: PathBuf,
        old_hash: Hash,
        new_hash: Hash,
    ) -> Self {
        Self {
            path: new_path,
            old_path: Some(old_path),
            old_hash: Some(old_hash),
            new_hash: Some(new_hash),
            kind: ChangeKind::Copied,
        }
    }

    /// Returns the path of the changed file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the old path if the file was renamed or copied.
    #[must_use]
    pub fn old_path(&self) -> Option<&Path> {
        self.old_path.as_deref()
    }

    /// Returns the old hash, if the file previously existed.
    #[must_use]
    pub const fn old_hash(&self) -> Option<Hash> {
        self.old_hash
    }

    /// Returns the new hash, if the file exists now.
    #[must_use]
    pub const fn new_hash(&self) -> Option<Hash> {
        self.new_hash
    }

    /// Returns the kind of change.
    #[must_use]
    pub const fn kind(&self) -> ChangeKind {
        self.kind
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

    /// Returns `true` if this is a rename.
    #[must_use]
    pub fn is_renamed(&self) -> bool {
        self.kind == ChangeKind::Renamed
    }

    /// Returns `true` if this is a copy.
    #[must_use]
    pub fn is_copied(&self) -> bool {
        self.kind == ChangeKind::Copied
    }
}

/// A collection of file deltas between two trees.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreeDelta {
    changes: Vec<FileDelta>,
}

impl TreeDelta {
    /// Creates an empty `TreeDelta`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }

    /// Creates a `TreeDelta` from a vector of `FileDelta`.
    #[must_use]
    pub const fn from_changes(changes: Vec<FileDelta>) -> Self {
        Self { changes }
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

    /// Returns the changes.
    #[must_use]
    pub fn changes(&self) -> &[FileDelta] {
        &self.changes
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
