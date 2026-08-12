//! Commit objects and associated metadata.
//!
//! This module defines [`Commit`] – a snapshot of the repository state –
//! and [`CommitMeta`] – a separate struct for optional timestamp and
//! encoding information. The split allows constructing a commit with
//! default metadata (zeroed timestamps) via [`Commit::new`], or with
//! explicit metadata via [`Commit::with_meta`].

use super::hash::Hash;
use super::user_id::UserID;

/// Optional metadata for a commit, typically obtained from the environment
/// at creation time.
///
/// Separating this data into its own struct avoids clutter in [`Commit`]
/// and makes it easy to apply default values when no explicit metadata is
/// supplied.
///
/// All fields are public so that callers can freely construct and inspect
/// the metadata.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::types::core::CommitMeta;
///
/// let meta = CommitMeta {
///     timestamp: 1_700_000_000,
///     timezone_offset: -300,
///     encoding: Some("utf-8".into()),
/// };
/// assert_eq!(meta.timestamp, 1_700_000_000);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct CommitMeta {
    /// Seconds since the Unix epoch.
    pub timestamp: i64,
    /// Offset from UTC in minutes (e.g., -300 for EST).
    pub timezone_offset: i16,
    /// Character encoding used for the commit message, if specified.
    pub encoding: Option<String>,
}

/// A commit object representing a point in the version history.
///
/// A commit captures the state of the repository at a specific moment by
/// pointing to the root [`Tree`](super::Tree) via `tree`, recording the
/// author and committer, and optionally linking to parent commits through
/// `parents`. The commit message describes the change.
///
/// # Why private fields?
///
/// All fields are private to enforce immutability after construction.
/// Once a commit is created, its identity (hash) is fixed; allowing
/// mutation would break the content-addressable model. Accessors provide
/// read-only access.
///
/// # Examples
///
/// Building a commit with default metadata:
///
/// ```
/// use libvctrl_handler::types::core::{Commit, Hash, UserID};
/// # use std::str::FromStr;
/// # let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
/// # let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
/// # let committer = UserID::new("Bob".into(), "bob@example.com".into()).unwrap();
/// let commit = Commit::new(
///     tree,
///     vec![],
///     author,
///     committer,
///     "Initial commit".into(),
/// );
/// assert_eq!(commit.message(), "Initial commit");
/// assert_eq!(commit.parents().len(), 0);
/// ```
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
    /// Creates a new commit with zeroed timestamp/offset and no encoding.
    ///
    /// This constructor is useful when metadata is not yet known or when
    /// the caller intends to apply metadata later. For full control, use
    /// [`with_meta`](Self::with_meta).
    ///
    /// # Arguments
    ///
    /// * `tree` - The root tree hash.
    /// * `parents` - Previous commit hashes (can be empty for initial commit).
    /// * `author` - The person who authored the changes.
    /// * `committer` - The person who committed the changes.
    /// * `message` - The commit message.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Commit, Hash, UserID};
    /// # use std::str::FromStr;
    /// # let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// # let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// # let committer = UserID::new("Bob".into(), "bob@example.com".into()).unwrap();
    /// let commit = Commit::new(
    ///     tree,
    ///     vec![tree], // parent is same tree for example
    ///     author,
    ///     committer,
    ///     "Second commit".into(),
    /// );
    /// assert_eq!(commit.timestamp(), 0);
    /// assert_eq!(commit.encoding(), None);
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

    /// Creates a new commit with the supplied metadata.
    ///
    /// This is the preferred constructor when timestamp, timezone offset,
    /// or encoding information is available (e.g., from the environment
    /// or a previous commit).
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Commit, CommitMeta, Hash, UserID};
    /// # use std::str::FromStr;
    /// # let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// # let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// # let committer = UserID::new("Bob".into(), "bob@example.com".into()).unwrap();
    /// let meta = CommitMeta {
    ///     timestamp: 1_700_000_000,
    ///     timezone_offset: 360,
    ///     encoding: Some("utf-8".into()),
    /// };
    /// let commit = Commit::with_meta(
    ///     tree,
    ///     vec![],
    ///     author,
    ///     committer,
    ///     "Commit with metadata".into(),
    ///     meta,
    /// );
    /// assert_eq!(commit.timestamp(), 1_700_000_000);
    /// assert_eq!(commit.encoding(), Some("utf-8"));
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

    /// Returns the root tree hash this commit points to.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Commit, Hash, UserID};
    /// # use std::str::FromStr;
    /// # let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// # let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// # let commit = Commit::new(tree, vec![], author.clone(), author, "msg".into());
    /// let root_tree = commit.tree();
    /// # let _ = root_tree;
    /// ```
    #[must_use]
    pub const fn tree(&self) -> &Hash {
        &self.tree
    }

    /// Returns a slice of parent commit hashes.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Commit, Hash, UserID};
    /// # use std::str::FromStr;
    /// # let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// # let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// # let parent = tree;
    /// # let commit = Commit::new(tree, vec![parent], author.clone(), author, "msg".into());
    /// let parents = commit.parents();
    /// assert_eq!(parents.len(), 1);
    /// ```
    #[must_use]
    pub fn parents(&self) -> &[Hash] {
        &self.parents
    }

    /// Returns the author of the changes.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Commit, Hash, UserID};
    /// # use std::str::FromStr;
    /// # let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// # let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// # let commit = Commit::new(tree, vec![], author.clone(), author, "msg".into());
    /// let author_ref = commit.author();
    /// assert_eq!(author_ref.name(), "Alice");
    /// ```
    #[must_use]
    pub const fn author(&self) -> &UserID {
        &self.author
    }

    /// Returns the committer who added the commit to the repository.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Commit, Hash, UserID};
    /// # use std::str::FromStr;
    /// # let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// # let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// # let committer = UserID::new("Bob".into(), "bob@example.com".into()).unwrap();
    /// # let commit = Commit::new(tree, vec![], author, committer.clone(), "msg".into());
    /// let committer_ref = commit.committer();
    /// assert_eq!(committer_ref.name(), "Bob");
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
    /// use libvctrl_handler::types::core::{Commit, Hash, UserID};
    /// # use std::str::FromStr;
    /// # let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// # let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// # let commit = Commit::new(tree, vec![], author.clone(), author, "Hello world".into());
    /// assert_eq!(commit.message(), "Hello world");
    /// ```
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the Unix timestamp (seconds since epoch) of the commit.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Commit, Hash, UserID};
    /// # use std::str::FromStr;
    /// # let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// # let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// # let commit = Commit::new(tree, vec![], author.clone(), author, "msg".into());
    /// assert_eq!(commit.timestamp(), 0);
    /// ```
    #[must_use]
    pub const fn timestamp(&self) -> i64 {
        self.timestamp
    }

    /// Returns the timezone offset from UTC in minutes.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Commit, Hash, UserID};
    /// # use std::str::FromStr;
    /// # let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// # let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// # let commit = Commit::new(tree, vec![], author.clone(), author, "msg".into());
    /// assert_eq!(commit.timezone_offset(), 0);
    /// ```
    #[must_use]
    pub const fn timezone_offset(&self) -> i16 {
        self.timezone_offset
    }

    /// Returns the character encoding of the commit message, if specified.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Commit, CommitMeta, Hash, UserID};
    /// # use std::str::FromStr;
    /// # let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// # let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// # let meta = CommitMeta { timestamp: 0, timezone_offset: 0, encoding: Some("utf-16".into()) };
    /// # let commit = Commit::with_meta(tree, vec![], author.clone(), author, "msg".into(), meta);
    /// assert_eq!(commit.encoding(), Some("utf-16"));
    /// ```
    #[must_use]
    pub fn encoding(&self) -> Option<&str> {
        self.encoding.as_deref()
    }
}
