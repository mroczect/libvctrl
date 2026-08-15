use std::path::PathBuf;

use crate::Hash;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conflict {
    pub path: PathBuf,

    pub ancestor_blob: Hash,

    pub our_blob: Hash,

    pub their_blob: Hash,
}

impl Conflict {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeResult {
    Success(Hash),

    Conflicts(Vec<Conflict>),
}

impl MergeResult {
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    #[must_use]
    pub const fn is_conflicts(&self) -> bool {
        matches!(self, Self::Conflicts(_))
    }

    #[must_use]
    pub fn conflicts(&self) -> Option<&[Conflict]> {
        match self {
            Self::Conflicts(conflicts) => Some(conflicts),
            Self::Success(_) => None,
        }
    }
}
