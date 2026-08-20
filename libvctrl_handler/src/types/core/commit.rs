use std::collections::HashSet;

use super::hash::Hash;
use super::user_id::UserID;
use crate::constants::{MAX_MESSAGE_LENGTH, MAX_PARENT_COUNT};
use crate::errors::VctrlError;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct CommitMeta {
    timestamp: i64,
    timezone_offset: i16,
    encoding: Option<String>,
}

impl CommitMeta {
    pub fn new(
        timestamp: i64,
        timezone_offset: i16,
        encoding: Option<String>,
    ) -> Result<Self, VctrlError> {
        if !(-1440..=1440).contains(&timezone_offset) {
            return Err(VctrlError::InvalidTimezoneOffset(timezone_offset));
        }
        Ok(Self {
            timestamp,
            timezone_offset,
            encoding,
        })
    }

    #[must_use]
    pub const fn timestamp(&self) -> i64 {
        self.timestamp
    }

    #[must_use]
    pub const fn timezone_offset(&self) -> i16 {
        self.timezone_offset
    }

    #[must_use]
    pub fn encoding(&self) -> Option<&str> {
        self.encoding.as_deref()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Commit {
    tree: Hash,
    parents: Vec<Hash>,
    author: UserID,
    committer: UserID,
    message: String,
    meta: CommitMeta,
}

impl Commit {
    pub fn new(
        tree: Hash,
        parents: Vec<Hash>,
        author: UserID,
        committer: UserID,
        message: String,
    ) -> Result<Self, VctrlError> {
        Self::with_meta(
            tree,
            parents,
            author,
            committer,
            message,
            CommitMeta::default(),
        )
    }

    pub fn with_meta(
        tree: Hash,
        parents: Vec<Hash>,
        author: UserID,
        committer: UserID,
        message: String,
        meta: CommitMeta,
    ) -> Result<Self, VctrlError> {
        let max_parents = usize::try_from(MAX_PARENT_COUNT).unwrap_or(usize::MAX);
        if parents.len() > max_parents {
            return Err(VctrlError::ExceededMaxSize(format!(
                "commit has {} parents, exceeding maximum of {MAX_PARENT_COUNT}",
                parents.len()
            )));
        }

        let max_len = usize::try_from(MAX_MESSAGE_LENGTH).unwrap_or(usize::MAX);
        if message.len() > max_len {
            return Err(VctrlError::ExceededMaxSize(format!(
                "message length exceeds maximum allowed length {MAX_MESSAGE_LENGTH}"
            )));
        }

        let mut seen = HashSet::new();
        for parent in &parents {
            if !seen.insert(*parent) {
                return Err(VctrlError::DuplicateParent);
            }
        }

        Ok(Self {
            tree,
            parents,
            author,
            committer,
            message,
            meta,
        })
    }

    #[must_use]
    pub const fn tree(&self) -> &Hash {
        &self.tree
    }

    #[must_use]
    pub fn parents(&self) -> &[Hash] {
        &self.parents
    }

    #[must_use]
    pub const fn author(&self) -> &UserID {
        &self.author
    }

    #[must_use]
    pub const fn committer(&self) -> &UserID {
        &self.committer
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub const fn meta(&self) -> &CommitMeta {
        &self.meta
    }
}
