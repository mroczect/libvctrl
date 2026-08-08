use crate::types::hash::Hash;
use crate::types::user_id::UserID;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct CommitMeta {
    pub timestamp: i64,
    pub timezone_offset: i16,
    pub encoding: Option<String>,
}

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
    #[must_use]
    pub const fn new(
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
