//! Commit and metadata types for the `libvctrl_handler` version control
//! contracts.
//!
//! # Purpose
//! A [`Commit`](crate::Commit) represents a snapshot of the entire repository
//! tree at a specific point in time, along with the history of how that
//! snapshot was created. It links a [`Tree`](crate::Tree) to its parent
//! commits, author, and committer information.
//!
//! # Design rationale
//! The metadata fields (`timestamp`, `timezone_offset`, `encoding`) are
//! grouped into [`CommitMeta`](crate::CommitMeta) to prevent the
//! [`Commit`](crate::Commit) constructors from having an excessive number of
//! arguments. This also allows the same metadata block to be reused across
//! multiple commits (for example, during a bulk import). The fields of
//! [`CommitMeta`](crate::CommitMeta) are public because the struct is a plain
//! data aggregate; it has no invariants to enforce.
//!
//! The [`Commit`](crate::Commit) struct itself keeps its fields private. A
//! commit is immutable once created; exposing mutable access would break the
//! cryptographic integrity of the history graph. All accessors return shared
//! references or copied scalars.

use crate::types::hash::Hash;
use crate::types::user_id::UserID;

/// Metadata associated with a commit or tag.
///
/// # Purpose
/// This struct groups together optional metadata fields (timestamp, timezone,
/// and encoding) so that the [`Commit`](crate::Commit) and
/// [`Tag`](crate::Tag) constructors do not need to take a large number of
/// arguments. It also allows the same metadata to be reused across multiple
/// items.
///
/// # Design rationale
/// The fields are public because `CommitMeta` is a plain data aggregate with
/// no invariants to enforce. It derives [`Default`](std::default::Default) to
/// allow easy construction with sensible defaults (timestamp `0`, offset `0`,
/// no encoding).
///
/// # Examples
///
/// ```
/// use libvctrl_handler::CommitMeta;
///
/// let meta = CommitMeta {
///     timestamp: 1_700_000_000,
///     timezone_offset: 120,
///     encoding: Some("UTF-8".to_string()),
/// };
/// assert_eq!(meta.timestamp, 1_700_000_000);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct CommitMeta {
    /// The Unix timestamp (seconds since epoch) when the commit or tag was created.
    pub timestamp: i64,
    /// The timezone offset in minutes from UTC. Positive is east, negative is west.
    pub timezone_offset: i16,
    /// The character encoding of the `message` field (e.g., `"UTF-8"`). `None` implies UTF-8 by default.
    pub encoding: Option<String>,
}

/// An immutable snapshot of a repository tree and its history.
///
/// # Purpose
/// A `Commit` is the primary node in the version control history graph. It
/// ties together a [`Tree`](crate::Tree) (the file contents), zero or more
/// parent commits (the history), and the [`UserID`](crate::UserID) of the
/// people who authored and committed the change.
///
/// # Design rationale
/// All fields are private to enforce immutability. The cryptographic hash of
/// a commit depends on every field; mutating any field after creation would
/// invalidate the [`Hash`](crate::Hash) that identifies this commit in the
/// [`ObjectStore`](crate::ObjectStore).
///
/// # Internal mechanism
/// The struct holds owned copies of all data. The [`parents`](Commit::parents)
/// field is a `Vec<Hash>` to support merge commits with multiple parents.
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
    /// Creates a new `Commit` with default metadata (timestamp `0`, offset `0`, no encoding).
    ///
    /// # Design rationale
    /// This is a `const fn` to allow compile-time construction of commit
    /// structures where possible, and to maintain API consistency with other
    /// constructors in the crate.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::{Commit, Hash, UserID};
    ///
    /// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let author = UserID::new("Alice".to_string(), "alice@example.com".to_string()).unwrap();
    /// let committer = UserID::new("Bob".to_string(), "bob@example.com".to_string()).unwrap();
    ///
    /// let commit = Commit::new(
    ///     tree,
    ///     Vec::new(),
    ///     author,
    ///     committer,
    ///     "Initial commit".to_string(),
    /// );
    ///
    /// assert_eq!(commit.timestamp(), 0);
    /// ```
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

    /// Creates a new `Commit` with explicit metadata.
    ///
    /// # Design rationale
    /// This constructor takes a [`CommitMeta`](crate::CommitMeta) struct to
    /// keep the argument list manageable and allow metadata reuse.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::{Commit, CommitMeta, Hash, UserID};
    ///
    /// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let author = UserID::new("Alice".to_string(), "alice@example.com".to_string()).unwrap();
    /// let committer = UserID::new("Bob".to_string(), "bob@example.com".to_string()).unwrap();
    /// let meta = CommitMeta {
    ///     timestamp: 1_700_000_000,
    ///     timezone_offset: 120,
    ///     encoding: Some("UTF-8".to_string()),
    /// };
    ///
    /// let commit = Commit::with_meta(
    ///     tree,
    ///     Vec::new(),
    ///     author,
    ///     committer,
    ///     "Initial commit".to_string(),
    ///     meta,
    /// );
    ///
    /// assert_eq!(commit.timestamp(), 1_700_000_000);
    /// ```
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

    /// Returns the hash of the tree this commit points to.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::{Commit, Hash, UserID};
    ///
    /// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let author = UserID::new("A".to_string(), "a@a.com".to_string()).unwrap();
    /// let committer = UserID::new("B".to_string(), "b@b.com".to_string()).unwrap();
    ///
    /// let commit = Commit::new(tree, Vec::new(), author, committer, "msg".to_string());
    /// assert_eq!(commit.tree(), &tree);
    /// ```
    #[must_use]
    pub const fn tree(&self) -> &Hash {
        &self.tree
    }

    /// Returns the hashes of the parent commits.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::{Commit, Hash, UserID};
    ///
    /// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let parent = Hash::from_bytes(&[1u8; 64]).unwrap();
    /// let author = UserID::new("A".to_string(), "a@a.com".to_string()).unwrap();
    /// let committer = UserID::new("B".to_string(), "b@b.com".to_string()).unwrap();
    ///
    /// let commit = Commit::new(tree, vec![parent], author, committer, "msg".to_string());
    /// assert_eq!(commit.parents().len(), 1);
    /// ```
    #[must_use]
    pub fn parents(&self) -> &[Hash] {
        &self.parents
    }

    /// Returns the author of the commit.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::{Commit, Hash, UserID};
    ///
    /// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let author = UserID::new("Alice".to_string(), "alice@example.com".to_string()).unwrap();
    /// let committer = UserID::new("Bob".to_string(), "bob@example.com".to_string()).unwrap();
    ///
    /// let commit = Commit::new(tree, Vec::new(), author.clone(), committer, "msg".to_string());
    /// assert_eq!(commit.author(), &author);
    /// ```
    #[must_use]
    pub const fn author(&self) -> &UserID {
        &self.author
    }

    /// Returns the committer of the commit.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::{Commit, Hash, UserID};
    ///
    /// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let author = UserID::new("Alice".to_string(), "alice@example.com".to_string()).unwrap();
    /// let committer = UserID::new("Bob".to_string(), "bob@example.com".to_string()).unwrap();
    ///
    /// let commit = Commit::new(tree, Vec::new(), author, committer.clone(), "msg".to_string());
    /// assert_eq!(commit.committer(), &committer);
    /// ```
    #[must_use]
    pub const fn committer(&self) -> &UserID {
        &self.committer
    }

    /// Returns the commit message.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::{Commit, Hash, UserID};
    ///
    /// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let author = UserID::new("A".to_string(), "a@a.com".to_string()).unwrap();
    /// let committer = UserID::new("B".to_string(), "b@b.com".to_string()).unwrap();
    ///
    /// let commit = Commit::new(tree, Vec::new(), author, committer, "Hello".to_string());
    /// assert_eq!(commit.message(), "Hello");
    /// ```
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the timestamp of the commit.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::{Commit, Hash, UserID};
    ///
    /// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let author = UserID::new("A".to_string(), "a@a.com".to_string()).unwrap();
    /// let committer = UserID::new("B".to_string(), "b@b.com".to_string()).unwrap();
    ///
    /// let commit = Commit::new(tree, Vec::new(), author, committer, "msg".to_string());
    /// assert_eq!(commit.timestamp(), 0);
    /// ```
    #[must_use]
    pub const fn timestamp(&self) -> i64 {
        self.timestamp
    }

    /// Returns the timezone offset of the commit.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::{Commit, Hash, UserID};
    ///
    /// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let author = UserID::new("A".to_string(), "a@a.com".to_string()).unwrap();
    /// let committer = UserID::new("B".to_string(), "b@b.com".to_string()).unwrap();
    ///
    /// let commit = Commit::new(tree, Vec::new(), author, committer, "msg".to_string());
    /// assert_eq!(commit.timezone_offset(), 0);
    /// ```
    #[must_use]
    pub const fn timezone_offset(&self) -> i16 {
        self.timezone_offset
    }

    /// Returns the character encoding of the commit message.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::{Commit, Hash, UserID};
    ///
    /// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let author = UserID::new("A".to_string(), "a@a.com".to_string()).unwrap();
    /// let committer = UserID::new("B".to_string(), "b@b.com".to_string()).unwrap();
    ///
    /// let commit = Commit::new(tree, Vec::new(), author, committer, "msg".to_string());
    /// assert_eq!(commit.encoding(), None);
    /// ```
    #[must_use]
    pub fn encoding(&self) -> Option<&str> {
        self.encoding.as_deref()
    }
}
