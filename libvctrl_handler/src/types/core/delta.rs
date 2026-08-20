use std::path::{Path, PathBuf};

use crate::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeKind {
    Added,

    Deleted,

    Modified,

    TypeChange,

    Renamed,

    Copied,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileDelta {
    path: PathBuf,
    old_path: Option<PathBuf>,
    old_hash: Option<Hash>,
    new_hash: Option<Hash>,
    kind: ChangeKind,
}

impl FileDelta {
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

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn old_path(&self) -> Option<&Path> {
        self.old_path.as_deref()
    }

    #[must_use]
    pub const fn old_hash(&self) -> Option<Hash> {
        self.old_hash
    }

    #[must_use]
    pub const fn new_hash(&self) -> Option<Hash> {
        self.new_hash
    }

    #[must_use]
    pub const fn kind(&self) -> ChangeKind {
        self.kind
    }

    #[must_use]
    pub fn is_added(&self) -> bool {
        self.kind == ChangeKind::Added
    }

    #[must_use]
    pub fn is_deleted(&self) -> bool {
        self.kind == ChangeKind::Deleted
    }

    #[must_use]
    pub fn is_modified(&self) -> bool {
        self.kind == ChangeKind::Modified
    }

    #[must_use]
    pub fn is_type_change(&self) -> bool {
        self.kind == ChangeKind::TypeChange
    }

    #[must_use]
    pub fn is_renamed(&self) -> bool {
        self.kind == ChangeKind::Renamed
    }

    #[must_use]
    pub fn is_copied(&self) -> bool {
        self.kind == ChangeKind::Copied
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreeDelta {
    changes: Vec<FileDelta>,
}

impl TreeDelta {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }

    #[must_use]
    pub const fn from_changes(changes: Vec<FileDelta>) -> Self {
        Self { changes }
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.changes.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, FileDelta> {
        self.changes.iter()
    }

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
