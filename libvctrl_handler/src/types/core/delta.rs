use std::path::PathBuf;

use crate::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeKind {
    Added,

    Deleted,

    Modified,

    TypeChange,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileDelta {
    pub path: PathBuf,

    pub old_hash: Option<Hash>,

    pub new_hash: Option<Hash>,

    pub kind: ChangeKind,
}

impl FileDelta {
    #[must_use]
    pub const fn new(path: PathBuf, kind: ChangeKind) -> Self {
        Self {
            path,
            old_hash: None,
            new_hash: None,
            kind,
        }
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
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreeDelta {
    pub changes: Vec<FileDelta>,
}

impl TreeDelta {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
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
