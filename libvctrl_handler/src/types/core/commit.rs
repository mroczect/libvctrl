//! Commit object representation.

use super::hash::Hash;
use super::user_id::UserID;

/// Metadata associated with a commit or tag.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct CommitMeta {
    /// Unix timestamp.
    pub timestamp: i64,
    /// Timezone offset in minutes.
    pub timezone_offset: i16,
    /// Optional character encoding.
    pub encoding: Option<String>,
}

/// A Git commit object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Commit {
    tree: Hash,
    parents: Vec<Hash>,
    author: UserID,
    committer: UserID,
    message: String,
    timestamp: i64,
    timezone_offset: i16,
    encoding: Option<String>,
}

impl Commit {
    /// Creates a new commit without timestamp metadata.
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn new(
        tree: Hash,
        parents: Vec<Hash>,
        author: UserID,
        committer: UserID,
        message: String,
    ) -> Self {
        Self {
            tree,
            parents,
            author,
            committer,
            message,
            timestamp: 0,
            timezone_offset: 0,
            encoding: None,
        }
    }

    /// Creates a new commit with timestamp metadata.
    #[must_use]
    pub fn with_meta(
        tree: Hash,
        parents: Vec<Hash>,
        author: UserID,
        committer: UserID,
        message: String,
        meta: CommitMeta,
    ) -> Self {
        Self {
            tree,
            parents,
            author,
            committer,
            message,
            timestamp: meta.timestamp,
            timezone_offset: meta.timezone_offset,
            encoding: meta.encoding,
        }
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

    /// Returns the commit timestamp.
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
