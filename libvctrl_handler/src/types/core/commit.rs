//! Commit object representation.

use super::hash::Hash;
use super::user_id::UserID;
use crate::constants::MAX_MESSAGE_LENGTH;
use crate::errors::VctrlError;
use std::collections::HashSet;

/// Metadata associated with a commit or tag.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct CommitMeta {
    timestamp: i64,
    timezone_offset: i16,
    encoding: Option<String>,
}

impl CommitMeta {
    /// Creates new commit metadata.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidTimezoneOffset`] if the offset is out of range.
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

    /// Returns the timestamp.
    #[must_use]
    pub const fn timestamp(&self) -> i64 {
        self.timestamp
    }

    /// Returns the timezone offset.
    #[must_use]
    pub const fn timezone_offset(&self) -> i16 {
        self.timezone_offset
    }

    /// Returns the encoding, if any.
    #[must_use]
    pub fn encoding(&self) -> Option<&str> {
        self.encoding.as_deref()
    }
}

/// A Git commit object.
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
    /// Creates a new commit without timestamp metadata.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::DuplicateParent`] if parents contain duplicates.
    /// Returns [`VctrlError::ExceededMaxSize`] if the message is too long.
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

    /// Creates a new commit with timestamp metadata.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if validation fails.
    pub fn with_meta(
        tree: Hash,
        parents: Vec<Hash>,
        author: UserID,
        committer: UserID,
        message: String,
        meta: CommitMeta,
    ) -> Result<Self, VctrlError> {
        if message.len() > MAX_MESSAGE_LENGTH as usize {
            return Err(VctrlError::ExceededMaxSize(format!(
                "message length exceeds maximum allowed length {MAX_MESSAGE_LENGTH}"
            )));
        }

        let mut seen = HashSet::new();
        for p in &parents {
            if !seen.insert(*p) {
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

    /// Returns the tree hash of this commit.
    #[must_use]
    pub const fn tree(&self) -> &Hash {
        &self.tree
    }

    /// Returns the parent commit hashes.
    #[must_use]
    pub fn parents(&self) -> &[Hash] {
        &self.parents
    }

    /// Returns the author information.
    #[must_use]
    pub const fn author(&self) -> &UserID {
        &self.author
    }

    /// Returns the committer information.
    #[must_use]
    pub const fn committer(&self) -> &UserID {
        &self.committer
    }

    /// Returns the commit message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the commit metadata.
    #[must_use]
    pub const fn meta(&self) -> &CommitMeta {
        &self.meta
    }
}
