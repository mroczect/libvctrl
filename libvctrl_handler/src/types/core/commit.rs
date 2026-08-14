//! Commit objects and associated metadata.
//!
//! # Purpose
//!
//! This module defines `Commit`, a snapshot of the repository state at a
//! particular point in history, and `CommitMeta`, a separate struct for
//! optional timestamp, timezone offset, and encoding information. The split
//! allows constructing a commit with default metadata (zeroed timestamps)
//! via `Commit::new`, or with explicit metadata via
//! `Commit::with_meta`.
//!
//! # Design Rationale
//!
//! Commits are central to version control. They represent immutable points
//! in the history graph. A commit captures:
//!
//! - The root `Tree` that describes the repository content.
//! - Zero or more parent commits, forming the history DAG.
//! - The identity of the author and committer.
//! - A human-readable message describing the change.
//! - Optional metadata such as timestamp and encoding.
//!
//! The object is immutable after construction. All fields are private and
//! accessible only through read-only accessor methods. This immutability is
//! essential because a commit's identity is its content hash; mutating any
//! field would change the hash and break the content-addressing model.
//!
//! # Relationship to Other Types
//!
//! - A `Commit` points to a `Tree` via its root tree hash.
//! - The author and committer are `UserID` instances.
//! - Parent commits are stored as a slice of `Hash` values.
//! - Metadata is encapsulated in `CommitMeta`, which is also used by
//!   `Tag`.
//!
//! # Memory Layout
//!
//! A `Commit` owns its fields: a `Hash` (64 bytes), a `Vec` of parent
//! hashes (heap-allocated), two `UserID` values (each owning two
//! `String`s), a `String` for the message, and a few scalar metadata
//! fields. The struct is not `Copy` because it owns heap-allocated data;
//! cloning performs a deep copy.
//!
//! # Examples
//!
//! Building a commit with default metadata:
//!
//! ```
//! use libvctrl_handler::types::core::{Commit, Hash, UserID};
//!
//! let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
//! let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
//! let committer = UserID::new("Bob".into(), "bob@example.com".into()).unwrap();
//!
//! let commit = Commit::new(
//!     tree,
//!     vec![],
//!     author,
//!     committer,
//!     "Initial commit".into(),
//! );
//!
//! assert_eq!(commit.message(), "Initial commit");
//! assert_eq!(commit.parents().len(), 0);
//! assert_eq!(commit.timestamp(), 0);
//! assert_eq!(commit.encoding(), None);
//! ```

use super::hash::Hash;
use super::user_id::UserID;

/// Optional metadata for a commit, typically obtained from the environment
/// at creation time.
///
/// # Purpose
///
/// This struct bundles the optional metadata fields that accompany a commit:
/// the creation timestamp, the timezone offset, and the character encoding
/// used for the commit message. Separating this data into its own struct
/// avoids clutter in `Commit` and makes it easy to apply default values
/// when no explicit metadata is supplied.
///
/// # Design Rationale
///
/// - All fields are public so that callers can freely construct and inspect
///   the metadata without accessor boilerplate.
/// - The struct derives `Default`, allowing zeroed timestamp, zero offset,
///   and no encoding as sensible defaults.
/// - Reusing this struct in both `Commit` and `Tag` avoids
///   duplicating the same three fields across multiple objects.
///
/// # Field Semantics
///
/// - `timestamp`: seconds since the Unix epoch.
/// - `timezone_offset`: offset from UTC in minutes (e.g., -300 for EST).
/// - `encoding`: character encoding used for the message, if specified.
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
///
/// assert_eq!(meta.timestamp, 1_700_000_000);
/// assert_eq!(meta.timezone_offset, -300);
/// assert_eq!(meta.encoding.as_deref(), Some("utf-8"));
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
/// # Purpose
///
/// A commit captures the state of the repository at a specific moment by
/// pointing to the root `Tree` via its `tree` hash,
/// recording the author and committer, and optionally linking to parent
/// commits through `parents`. The commit message describes the change.
///
/// # Why private fields?
///
/// All fields are private to enforce immutability after construction.
/// Once a commit is created, its identity (hash) is fixed; allowing
/// mutation would break the content-addressable model. Accessors provide
/// read-only access to each field.
///
/// # Design Rationale
///
/// - The constructor `Commit::new` provides a minimal commit with zeroed
///   timestamps and no encoding, suitable for cases where metadata is not
///   yet known.
/// - The alternative constructor `Commit::with_meta` accepts a
///   `CommitMeta` for full control over timestamp, timezone offset, and
///   encoding.
/// - Parent commits are stored in a `Vec` to allow zero or more parents.
///   An initial commit has an empty parent list; merge commits have multiple
///   parents.
/// - The author and committer are separate identities to support cases
///   where the person who wrote the changes differs from the person who
///   applied them.
///
/// # Immutability
///
/// The struct is immutable after construction. No mutable accessors are
/// provided. This is a deliberate design choice to preserve the integrity of
/// the commit's content hash.
///
/// # Examples
///
/// Building a commit with default metadata:
///
/// ```
/// use libvctrl_handler::types::core::{Commit, Hash, UserID};
///
/// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
/// let committer = UserID::new("Bob".into(), "bob@example.com".into()).unwrap();
///
/// let commit = Commit::new(
///     tree,
///     vec![],
///     author,
///     committer,
///     "Initial commit".into(),
/// );
///
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
    /// `with_meta`.
    ///
    /// # Arguments
    ///
    /// * `tree` - The root tree hash.
    /// * `parents` - Previous commit hashes (can be empty for initial commit).
    /// * `author` - The person who authored the changes.
    /// * `committer` - The person who committed the changes.
    /// * `message` - The commit message.
    ///
    /// # Returns
    ///
    /// A new `Commit` instance with `timestamp = 0`, `timezone_offset = 0`,
    /// and `encoding = None`.
    ///
    /// # Why not validate?
    ///
    /// This constructor intentionally does not validate message length or
    /// parent count. Validation, if required, is the responsibility of
    /// higher-level builders or encoders. Keeping the constructor simple
    /// makes it suitable for rapid prototyping and testing.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Commit, Hash, UserID};
    ///
    /// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// let committer = UserID::new("Bob".into(), "bob@example.com".into()).unwrap();
    ///
    /// let commit = Commit::new(
    ///     tree,
    ///     vec![tree],
    ///     author,
    ///     committer,
    ///     "Second commit".into(),
    /// );
    ///
    /// assert_eq!(commit.timestamp(), 0);
    /// assert_eq!(commit.timezone_offset(), 0);
    /// assert_eq!(commit.encoding(), None);
    /// assert_eq!(commit.parents().len(), 1);
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
    /// # Arguments
    ///
    /// * `tree` - The root tree hash.
    /// * `parents` - Previous commit hashes (can be empty for initial commit).
    /// * `author` - The person who authored the changes.
    /// * `committer` - The person who committed the changes.
    /// * `message` - The commit message.
    /// * `meta` - A `CommitMeta` containing timestamp, timezone offset,
    ///   and encoding.
    ///
    /// # Returns
    ///
    /// A new `Commit` instance with the provided metadata.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Commit, CommitMeta, Hash, UserID};
    ///
    /// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// let committer = UserID::new("Bob".into(), "bob@example.com".into()).unwrap();
    /// let meta = CommitMeta {
    ///     timestamp: 1_700_000_000,
    ///     timezone_offset: 360,
    ///     encoding: Some("utf-8".into()),
    /// };
    ///
    /// let commit = Commit::with_meta(
    ///     tree,
    ///     vec![],
    ///     author,
    ///     committer,
    ///     "Commit with metadata".into(),
    ///     meta,
    /// );
    ///
    /// assert_eq!(commit.timestamp(), 1_700_000_000);
    /// assert_eq!(commit.timezone_offset(), 360);
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
    /// # Returns
    ///
    /// A reference to the `Hash` of the root tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Commit, Hash, UserID};
    ///
    /// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// let commit = Commit::new(tree, vec![], author.clone(), author, "msg".into());
    ///
    /// assert_eq!(commit.tree(), &tree);
    /// ```
    #[must_use]
    pub const fn tree(&self) -> &Hash {
        &self.tree
    }

    /// Returns a slice of parent commit hashes.
    ///
    /// # Returns
    ///
    /// A reference to the internal `Vec` of parent hashes as a slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Commit, Hash, UserID};
    ///
    /// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// let commit = Commit::new(tree, vec![tree], author.clone(), author, "msg".into());
    ///
    /// let parents = commit.parents();
    /// assert_eq!(parents.len(), 1);
    /// assert_eq!(parents[0], tree);
    /// ```
    #[must_use]
    pub fn parents(&self) -> &[Hash] {
        &self.parents
    }

    /// Returns the author of the changes.
    ///
    /// # Returns
    ///
    /// A reference to the `UserID` of the author.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Commit, Hash, UserID};
    ///
    /// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// let commit = Commit::new(tree, vec![], author.clone(), author, "msg".into());
    ///
    /// assert_eq!(commit.author().name(), "Alice");
    /// assert_eq!(commit.author().email(), "alice@example.com");
    /// ```
    #[must_use]
    pub const fn author(&self) -> &UserID {
        &self.author
    }

    /// Returns the committer who added the commit to the repository.
    ///
    /// # Returns
    ///
    /// A reference to the `UserID` of the committer.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Commit, Hash, UserID};
    ///
    /// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// let committer = UserID::new("Bob".into(), "bob@example.com".into()).unwrap();
    /// let commit = Commit::new(tree, vec![], author, committer.clone(), "msg".into());
    ///
    /// assert_eq!(commit.committer().name(), "Bob");
    /// assert_eq!(commit.committer().email(), "bob@example.com");
    /// ```
    #[must_use]
    pub const fn committer(&self) -> &UserID {
        &self.committer
    }

    /// Returns the commit message.
    ///
    /// # Returns
    ///
    /// A string slice containing the commit message.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Commit, Hash, UserID};
    ///
    /// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// let commit = Commit::new(tree, vec![], author.clone(), author, "Hello world".into());
    ///
    /// assert_eq!(commit.message(), "Hello world");
    /// ```
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the Unix timestamp (seconds since epoch) of the commit.
    ///
    /// # Returns
    ///
    /// The timestamp as an `i64`.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Commit, Hash, UserID};
    ///
    /// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// let commit = Commit::new(tree, vec![], author.clone(), author, "msg".into());
    ///
    /// assert_eq!(commit.timestamp(), 0);
    /// ```
    #[must_use]
    pub const fn timestamp(&self) -> i64 {
        self.timestamp
    }

    /// Returns the timezone offset from UTC in minutes.
    ///
    /// # Returns
    ///
    /// The offset as an `i16`. Positive values indicate east of UTC,
    /// negative values indicate west.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Commit, Hash, UserID};
    ///
    /// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// let commit = Commit::new(tree, vec![], author.clone(), author, "msg".into());
    ///
    /// assert_eq!(commit.timezone_offset(), 0);
    /// ```
    #[must_use]
    pub const fn timezone_offset(&self) -> i16 {
        self.timezone_offset
    }

    /// Returns the character encoding of the commit message, if specified.
    ///
    /// # Returns
    ///
    /// An `Option<&str>` containing the encoding name if present.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Commit, CommitMeta, Hash, UserID};
    ///
    /// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// let meta = CommitMeta {
    ///     timestamp: 0,
    ///     timezone_offset: 0,
    ///     encoding: Some("utf-16".into()),
    /// };
    /// let commit = Commit::with_meta(tree, vec![], author.clone(), author, "msg".into(), meta);
    ///
    /// assert_eq!(commit.encoding(), Some("utf-16"));
    /// ```
    #[must_use]
    pub fn encoding(&self) -> Option<&str> {
        self.encoding.as_deref()
    }
}
