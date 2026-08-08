//! # Commit – Snapshot of Repository State
//!
//! The `Commit` type is the core version‑control object. It records a point‑in‑time
//! snapshot of the repository, linking to a tree (the actual directory structure)
//! and one or more parent commits that form the history.
//!
//! ## Commit Graph
//!
//! Commits form a **directed acyclic graph** (DAG) via parent pointers:
//!
//! - A **root commit** has no parents (the first commit in a repository).
//! - A **normal commit** has exactly one parent (linear history).
//! - A **merge commit** has two or more parents (combining branches).
//! - An **octopus merge** has more than two parents (rare, but supported).
//!
//! ## Fields
//!
//! | Field | Description |
//! |-------|-------------|
//! | `tree` | Hash of the root [`Tree`](crate::Tree) object describing the directory state |
//! | `parents` | List of parent commit hashes (usually one; empty for root commits) |
//! | `author` | Person who authored the changes ([`UserID`](crate::UserID)) |
//! | `committer` | Person who committed the changes (often same as author) |
//! | `message` | Human‑readable description of the changes |
//! | `timestamp` | Unix timestamp when the commit was created |
//! | `timezone_offset` | Offset from UTC in minutes |
//! | `encoding` | Text encoding of the message (e.g., `"UTF-8"`) |
//!
//! ## Construction
//!
//! There are two ways to create a commit:
//!
//! 1. [`Commit::new`] – uses default metadata (timestamp 0, no encoding).
//! 2. [`Commit::with_meta`] – uses an explicit [`CommitMeta`] struct.
//!
//! ## Validation
//!
//! Unlike tree entries and tags, commit **names** are not validated because
//! commits do not have names – they are identified by their hash. However,
//! the commit **message** is limited to [`MAX_MESSAGE_LENGTH`](crate::MAX_MESSAGE_LENGTH)
//! (1 MiB) by decoders. There is no length restriction at construction time.
//!
//! ## Relationship to Other Objects
//!
//! ```
//! Commit ──tree──► Tree ──entries──► TreeEntry ──hash──► Blob
//!    │                                                    │
//!    └──parents──► Commit ──...                           │
//!                    │                                    │
//!                    └────────────────────────────────────┘
//! ```
//!
//! - A `Commit` points to a **tree** (the root directory).
//! - That `Tree` contains entries that point to **blobs** or nested **trees**.
//! - A `Commit` can point to **parent commits**, forming the history graph.
//!
//! # Examples
//!
//! ## Single‑Parent Commit (Regular)
//!
//! ```rust
//! use libvctrl_handler::{Commit, Hash, UserID, CommitMeta, HASH_LENGTH};
//!
//! # let tree_hash = Hash::from_bytes(&[0x33; HASH_LENGTH]).unwrap();
//! # let parent_hash = Hash::from_bytes(&[0x44; HASH_LENGTH]).unwrap();
//! # let author = UserID::new("Bob".into(), "bob@example.com".into()).unwrap();
//!
//! let commit = Commit::new(
//!     tree_hash,
//!     vec![parent_hash],
//!     author.clone(),
//!     author.clone(),
//!     "Fix bug #42".into(),
//! );
//!
//! assert_eq!(commit.parents().len(), 1);
//! assert_eq!(commit.timestamp(), 0); // default metadata
//! ```
//!
//! ## Initial Commit (No Parents)
//!
//! ```rust
//! # use libvctrl_handler::*;
//! # let tree_hash = Hash::from_bytes(&[0x55; HASH_LENGTH]).unwrap();
//! # let user = UserID::new("Alice".into(), "alice@e.com".into()).unwrap();
//! let initial = Commit::new(tree_hash, vec![], user.clone(), user, "Initial commit".into());
//! assert!(initial.parents().is_empty());
//! ```
//!
//! ## Merge Commit (Two Parents)
//!
//! ```rust
//! # use libvctrl_handler::*;
//! # let tree_hash = Hash::from_bytes(&[0x66; HASH_LENGTH]).unwrap();
//! # let p1 = Hash::from_bytes(&[0x77; HASH_LENGTH]).unwrap();
//! # let p2 = Hash::from_bytes(&[0x88; HASH_LENGTH]).unwrap();
//! # let author = UserID::new("Alice".into(), "alice@e.com".into()).unwrap();
//! let merge = Commit::new(tree_hash, vec![p1, p2], author.clone(), author, "Merge branch 'feature'".into());
//! assert_eq!(merge.parents().len(), 2);
//! ```
//!
//! ## With Explicit Metadata
//!
//! ```rust
//! # use libvctrl_handler::*;
//! # let tree_hash = Hash::from_bytes(&[0x33; HASH_LENGTH]).unwrap();
//! # let parent_hash = Hash::from_bytes(&[0x44; HASH_LENGTH]).unwrap();
//! # let author = UserID::new("Bob".into(), "bob@example.com".into()).unwrap();
//! let meta = CommitMeta {
//!     timestamp: 1672531200,
//!     timezone_offset: 0,
//!     encoding: Some("UTF-8".into()),
//! };
//! let commit = Commit::with_meta(
//!     tree_hash,
//!     vec![parent_hash],
//!     author.clone(),
//!     author,
//!     "Fix bug #42".into(),
//!     meta,
//! );
//! assert_eq!(commit.timestamp(), 1672531200);
//! assert_eq!(commit.encoding(), Some("UTF-8"));
//! ```
//!
//! ## Accessing Commit Data
//!
//! ```rust
//! # use libvctrl_handler::*;
//! # let tree_hash = Hash::from_bytes(&[0x33; HASH_LENGTH]).unwrap();
//! # let parent_hash = Hash::from_bytes(&[0x44; HASH_LENGTH]).unwrap();
//! # let author = UserID::new("Bob".into(), "bob@example.com".into()).unwrap();
//! let commit = Commit::new(tree_hash, vec![parent_hash], author.clone(), author.clone(), "Bug fix".into());
//!
//! assert_eq!(commit.tree(), &tree_hash);
//! assert_eq!(commit.parents(), &[parent_hash]);
//! assert_eq!(commit.author().name(), "Bob");
//! assert_eq!(commit.committer().email(), "bob@example.com");
//! assert_eq!(commit.message(), "Bug fix");
//! ```
//!
//! # Serialization
//!
//! Commits are typically encoded to bytes using an [`Encoder`](crate::Encoder)
//! before storage. The reference binary format includes all fields in a
//! deterministic order. Decoders must enforce the message length limit and
//! reject malformed data.
//!
//! # Thread Safety
//!
//! `Commit` is `Send + Sync` because it holds no mutable state. It is safe to
//! share across threads.

use crate::types::hash::Hash;
use crate::types::user_id::UserID;

/// Optional metadata for [`Commit`] and [`Tag`] objects.
///
/// Bundles timestamp, timezone offset, and text encoding so that constructors
/// can accept a single metadata argument instead of many individual parameters.
///
/// # Default
///
/// `CommitMeta::default()` returns `timestamp: 0`, `timezone_offset: 0`,
/// `encoding: None`. This is what `Commit::new` and `Tag::new` use internally.
///
/// # Example
///
/// ```rust
/// use libvctrl_handler::CommitMeta;
///
/// let meta = CommitMeta {
///     timestamp: 1672531200,   // 2023-01-01T00:00:00 UTC
///     timezone_offset: 0,
///     encoding: Some("UTF-8".into()),
/// };
///
/// let default_meta = CommitMeta::default();
/// assert_eq!(default_meta.timestamp, 0);
/// assert!(default_meta.encoding.is_none());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct CommitMeta {
    /// Unix timestamp (seconds since the Unix epoch).  `0` means "not set".
    pub timestamp: i64,
    /// Timezone offset in minutes east of UTC.  `0` means "not set".
    pub timezone_offset: i16,
    /// Text encoding of the message (e.g., `"UTF-8"`).  `None` means "not specified".
    pub encoding: Option<String>,
}

/// A commit object – a snapshot of the repository at a point in time.
///
/// Records the root tree, parent commit(s), author, committer, a
/// human‑readable message, and optional metadata ([`CommitMeta`]).
///
/// # Construction
///
/// - [`Commit::new`] creates a commit with default metadata.
/// - [`Commit::with_meta`] accepts explicit [`CommitMeta`].
///
/// # Example (single‑parent commit)
///
/// ```rust
/// use libvctrl_handler::{Commit, Hash, UserID, HASH_LENGTH, CommitMeta};
///
/// let tree_hash = Hash::from_bytes(&[0x33; HASH_LENGTH]).unwrap();
/// let parent_hash = Hash::from_bytes(&[0x44; HASH_LENGTH]).unwrap();
/// let author = UserID::new("Bob".into(), "bob@example.com".into()).unwrap();
///
/// let commit = Commit::new(
///     tree_hash,
///     vec![parent_hash],
///     author.clone(),
///     author.clone(),
///     "Fix bug #42".into(),
/// );
///
/// // With metadata
/// let meta = CommitMeta {
///     timestamp: 1672531200,
///     timezone_offset: 0,
///     encoding: Some("UTF-8".into()),
/// };
/// let commit2 = Commit::with_meta(
///     tree_hash,
///     vec![parent_hash],
///     author.clone(),
///     author.clone(),
///     "Fix bug #42".into(),
///     meta,
/// );
/// ```
///
/// # Example (initial commit)
///
/// ```rust
/// # use libvctrl_handler::*;
/// let tree = Hash::from_bytes(&[0x55; 64]).unwrap();
/// let user = UserID::new("Alice".into(), "alice@e.com".into()).unwrap();
///
/// let initial = Commit::new(tree, vec![], user.clone(), user, "init".into());
/// assert!(initial.parents().is_empty());
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
    /// Creates a new `Commit` with default metadata (timestamp 0, no encoding).
    ///
    /// This is the simplest way to create a commit. For setting explicit
    /// timestamp or encoding, use [`with_meta`](Self::with_meta).
    ///
    /// # Arguments
    ///
    /// * `tree` – The root tree hash for this commit.
    /// * `parents` – A vector of parent commit hashes. An empty vector means
    ///   this is a root commit.
    /// * `author` – The person who authored the changes.
    /// * `committer` – The person who committed the changes.
    /// * `message` – The commit message (human‑readable description).
    ///
    /// # Note
    ///
    /// The message length is **not** validated here; decoders enforce
    /// [`MAX_MESSAGE_LENGTH`](crate::MAX_MESSAGE_LENGTH).
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

    /// Creates a new `Commit` with explicit metadata.
    ///
    /// This method allows full control over the commit metadata.
    ///
    /// # Arguments
    ///
    /// * `tree` – The root tree hash.
    /// * `parents` – A vector of parent commit hashes.
    /// * `author` – The author.
    /// * `committer` – The committer.
    /// * `message` – The commit message.
    /// * `meta` – The [`CommitMeta`] struct containing timestamp, timezone, and encoding.
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

    /// Returns the root tree hash.
    ///
    /// The tree hash points to a [`Tree`](crate::Tree) object that represents
    /// the entire directory structure at the time of this commit.
    #[must_use]
    pub const fn tree(&self) -> &Hash {
        &self.tree
    }

    /// Returns a reference to the parent hashes.
    ///
    /// The length of the returned slice indicates the type of commit:
    /// - 0 parents → root commit
    /// - 1 parent → regular commit
    /// - 2+ parents → merge commit (or octopus merge)
    #[must_use]
    pub fn parents(&self) -> &[Hash] {
        &self.parents
    }

    /// Returns the author.
    ///
    /// The author is the person who originally wrote the changes.
    #[must_use]
    pub const fn author(&self) -> &UserID {
        &self.author
    }

    /// Returns the committer.
    ///
    /// The committer is the person who applied the changes to the repository.
    /// In many cases, this is the same as the author.
    #[must_use]
    pub const fn committer(&self) -> &UserID {
        &self.committer
    }

    /// Returns the commit message.
    ///
    /// This is a human‑readable description of the changes. It may contain
    /// multiple lines and is typically used in `git log`‑like outputs.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Unix timestamp (seconds since epoch). 0 if not set.
    ///
    /// A value of 0 indicates that the timestamp was not explicitly set
    /// (default metadata). This is useful for testing or when the exact
    /// time is not important.
    #[must_use]
    pub const fn timestamp(&self) -> i64 {
        self.timestamp
    }

    /// Timezone offset in minutes east of UTC. 0 if not set.
    ///
    /// Positive values are east of UTC, negative values are west of UTC.
    /// A value of 0 means either UTC or "not set".
    #[must_use]
    pub const fn timezone_offset(&self) -> i16 {
        self.timezone_offset
    }

    /// Encoding (e.g., "UTF-8") if set.
    ///
    /// This indicates the character encoding of the commit message.
    /// `None` means the encoding is not specified (interpret as UTF-8).
    #[must_use]
    pub fn encoding(&self) -> Option<&str> {
        self.encoding.as_deref()
    }
}
