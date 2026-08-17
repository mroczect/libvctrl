//! Builder for constructing [`Commit`] objects with a fluent, type-safe API.
//!
//! # Why this module exists
//!
//! A [`Commit`] aggregates several mandatory pieces of metadata: a tree hash,
//! one or more parent hashes, author and committer identities, a message, and
//! optional metadata such as timestamp and encoding. Direct construction would
//! force every caller to provide all fields at once, even when they are built
//! incrementally or derived from different sources. The builder pattern solves
//! this by separating field assignment from final validation.
//!
//! # How it works
//!
//! The builder stores each field as an `Option` (or a `Vec` for parents) and
//! consumes `self` on every setter, returning `Self`. This ensures that each
//! setter is used exactly once in a chain and that the builder cannot be reused
//! after partial construction. The final [`build`](CommitBuilder::build)
//! method extracts all required fields, reports a descriptive [`VctrlError`]
//! if any are missing, and delegates to either [`Commit::with_meta`] or
//! [`Commit::new`] depending on whether metadata was supplied.
//!
//! # Examples
//!
//! ```
//! use libvctrl_core::object::CommitBuilder;
//! use libvctrl_handler::{Hash, UserID};
//!
//! let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
//! let author = UserID::new("Alice".to_owned(), "alice@example.com".to_owned()).unwrap();
//! let committer = UserID::new("Bob".to_owned(), "bob@example.com".to_owned()).unwrap();
//!
//! let commit = CommitBuilder::new()
//!     .tree(tree)
//!     .author(author)
//!     .committer(committer)
//!     .message("Initial commit")
//!     .build()
//!     .unwrap();
//!
//! assert_eq!(commit.message(), "Initial commit");
//! ```

use libvctrl_handler::{Commit, CommitMeta, Hash, UserID, VctrlError};

/// A builder for creating [`Commit`] objects.
///
/// # Design rationale
///
/// This type follows the *consuming builder* pattern. Each setter takes `self`
/// by value and returns `Self`, which makes the builder single-use and prevents
/// accidental reuse of a partially configured builder. Fields are stored
/// internally as `Option` (or a `Vec` for parents) because the builder must
/// remain `Default` while allowing the final [`build`](CommitBuilder::build)
/// to distinguish between “not provided” and “explicitly set to `None`”.
///
/// The struct is `#[derive(Default)]` so that callers may start from
/// `CommitBuilder::default()` if they prefer, but the explicit
/// [`new`](CommitBuilder::new) constructor is provided for clarity.
///
/// # Examples
///
/// Basic construction with all required fields:
///
/// ```
/// # use libvctrl_core::object::CommitBuilder;
/// # use libvctrl_handler::{Hash, UserID};
/// # let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
/// # let author = UserID::new("A".into(), "a@b.c".into()).unwrap();
/// # let committer = author.clone();
/// let commit = CommitBuilder::new()
///     .tree(tree)
///     .author(author)
///     .committer(committer)
///     .message("Initial commit")
///     .build()
///     .unwrap();
///
/// assert!(commit.parents().is_empty());
/// ```
#[derive(Debug, Default)]
pub struct CommitBuilder {
    tree: Option<Hash>,
    parents: Vec<Hash>,
    author: Option<UserID>,
    committer: Option<UserID>,
    message: Option<String>,
    meta: Option<CommitMeta>,
}

impl CommitBuilder {
    /// Creates a new `CommitBuilder` with no fields set.
    ///
    /// # Why this is `const`
    ///
    /// Marking the constructor as `const fn` allows the builder to be created
    /// in constant contexts and gives the compiler more opportunities for
    /// compile-time evaluation. The returned builder is a plain value on the
    /// stack with all `Option` fields set to `None` and the `parents` vector
    /// empty; no heap allocation occurs until the first `parent` call or
    /// message assignment.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_core::object::CommitBuilder;
    /// let builder = CommitBuilder::new();
    /// // builder is empty; calling build() now would fail with a missing-field error
    /// assert!(builder.build().is_err());
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {
            tree: None,
            parents: Vec::new(),
            author: None,
            committer: None,
            message: None,
            meta: None,
        }
    }

    /// Sets the tree hash for the commit.
    ///
    /// The tree hash points to the root tree object that represents the
    /// snapshot of the project at the time of the commit.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_core::object::CommitBuilder;
    /// # use libvctrl_handler::Hash;
    /// # let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let builder = CommitBuilder::new().tree(tree);
    /// assert!(builder.build().is_err()); // other fields still missing
    /// ```
    #[must_use]
    pub const fn tree(mut self, tree: Hash) -> Self {
        self.tree = Some(tree);
        self
    }

    /// Adds a parent commit hash.
    ///
    /// This method may be called multiple times to create a commit with
    /// multiple parents (e.g., a merge commit). Parents are stored in the
    /// order they are added, preserving the caller’s intended ordering for
    /// serialization.
    ///
    /// # Examples
    ///
    /// Adding two parents:
    ///
    /// ```
    /// # use libvctrl_core::object::CommitBuilder;
    /// # use libvctrl_handler::Hash;
    /// # let parent1 = Hash::from_bytes(&[1u8; 64]).unwrap();
    /// # let parent2 = Hash::from_bytes(&[2u8; 64]).unwrap();
    /// let builder = CommitBuilder::new()
    ///     .parent(parent1)
    ///     .parent(parent2);
    /// // Use builder further or build after setting other fields
    /// ```
    #[must_use]
    pub fn parent(mut self, parent: Hash) -> Self {
        self.parents.push(parent);
        self
    }

    /// Sets the author of the commit.
    ///
    /// The author is the person who originally wrote the changes, which may
    /// differ from the committer (for example, when applying a patch).
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_core::object::CommitBuilder;
    /// # use libvctrl_handler::UserID;
    /// # let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// let builder = CommitBuilder::new().author(author);
    /// assert!(builder.build().is_err()); // tree and committer still missing
    /// ```
    #[must_use]
    pub fn author(mut self, author: UserID) -> Self {
        self.author = Some(author);
        self
    }

    /// Sets the committer of the commit.
    ///
    /// The committer is the person who created the commit object. In simple
    /// workflows the author and committer are identical, but they are kept
    /// separate to preserve Git’s distinction.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_core::object::CommitBuilder;
    /// # use libvctrl_handler::UserID;
    /// # let committer = UserID::new("Bob".into(), "bob@example.com".into()).unwrap();
    /// let builder = CommitBuilder::new().committer(committer);
    /// assert!(builder.build().is_err()); // tree and author still missing
    /// ```
    #[must_use]
    pub fn committer(mut self, committer: UserID) -> Self {
        self.committer = Some(committer);
        self
    }

    /// Sets the commit message.
    ///
    /// The method accepts any type that implements `Into<String>`, including
    /// `&str`, `String`, and `Cow<str>`, making call sites ergonomic.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_core::object::CommitBuilder;
    /// let builder = CommitBuilder::new().message("Initial commit");
    /// // The message is stored internally as a String.
    /// assert!(builder.build().is_err()); // other required fields missing
    /// ```
    #[must_use]
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    /// Sets the optional commit metadata.
    ///
    /// Metadata includes the timestamp, timezone offset, and optional character
    /// encoding. If this method is not called, [`build`](CommitBuilder::build)
    /// delegates to [`Commit::new`], which uses default metadata.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_core::object::CommitBuilder;
    /// # use libvctrl_handler::CommitMeta;
    /// # let meta = CommitMeta::new(1_700_000_000, 0, None).unwrap();
    /// let builder = CommitBuilder::new().meta(meta);
    /// assert!(builder.build().is_err()); // other required fields missing
    /// ```
    #[must_use]
    pub fn meta(mut self, meta: CommitMeta) -> Self {
        self.meta = Some(meta);
        self
    }

    /// Builds the [`Commit`] object after validating all required fields.
    ///
    /// # How it works
    ///
    /// The method checks the four mandatory fields (`tree`, `author`,
    /// `committer`, and `message`) in order. If any is missing, it returns a
    /// [`VctrlError::Other`] with a descriptive message and does not allocate
    /// a commit. If all mandatory fields are present, it constructs the
    /// [`Commit`] by calling [`Commit::with_meta`] when metadata was supplied,
    /// or [`Commit::new`] otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::Other`] if any of the required fields is missing:
    /// - `tree`
    /// - `author`
    /// - `committer`
    /// - `message`
    ///
    /// Also returns any [`VctrlError`] produced by the underlying
    /// [`Commit::new`] or [`Commit::with_meta`] validation.
    ///
    /// # Examples
    ///
    /// Successful build:
    ///
    /// ```
    /// # use libvctrl_core::object::CommitBuilder;
    /// # use libvctrl_handler::{Hash, UserID};
    /// # let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// # let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// # let committer = UserID::new("Bob".into(), "bob@example.com".into()).unwrap();
    /// let commit = CommitBuilder::new()
    ///     .tree(tree)
    ///     .author(author)
    ///     .committer(committer)
    ///     .message("Initial commit")
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(commit.message(), "Initial commit");
    /// ```
    ///
    /// Missing field error:
    ///
    /// ```
    /// # use libvctrl_core::object::CommitBuilder;
    /// let result = CommitBuilder::new().build();
    /// assert!(result.is_err());
    /// ```
    pub fn build(self) -> Result<Commit, VctrlError> {
        let tree = self
            .tree
            .ok_or_else(|| VctrlError::Other("tree is required".into()))?;
        let author = self
            .author
            .ok_or_else(|| VctrlError::Other("author is required".into()))?;
        let committer = self
            .committer
            .ok_or_else(|| VctrlError::Other("committer is required".into()))?;
        let message = self
            .message
            .ok_or_else(|| VctrlError::Other("message is required".into()))?;

        if let Some(meta) = self.meta {
            Commit::with_meta(tree, self.parents, author, committer, message, meta)
        } else {
            Commit::new(tree, self.parents, author, committer, message)
        }
    }
}
