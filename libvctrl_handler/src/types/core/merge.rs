














use std::path::{Path, PathBuf};

use crate::Hash;





























#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conflict {
    path: PathBuf,
    ancestor_blob: Hash,
    our_blob: Hash,
    their_blob: Hash,
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

    
    
    
    
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    
    
    
    
    
    #[must_use]
    pub const fn ancestor_blob(&self) -> Hash {
        self.ancestor_blob
    }

    
    
    
    
    
    #[must_use]
    pub const fn our_blob(&self) -> Hash {
        self.our_blob
    }

    
    
    
    
    
    #[must_use]
    pub const fn their_blob(&self) -> Hash {
        self.their_blob
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
