//! Builder pattern for constructing [`Commit`](libvctrl_handler::Commit) objects.
//!
//! # Purpose
//! This module provides the [`CommitBuilder`], an ergonomic utility for
//! incrementally assembling version control commits. It solves the "telescoping
//! constructor" problem that arises from the `Commit` struct having many fields,
//! some required and some optional.
//!
//! # Design rationale
//! - **Required vs. Optional Separation**: The builder enforces that mandatory
//!   fields (`tree`, `author`, `committer`, `message`) are provided before
//!   construction. It returns a `Result` during `build()` to gracefully handle
//!   missing data, rather than panicking.
//! - **Method Chaining**: The builder consumes and returns `self` by value,
//!   enabling a fluent API. This makes commit creation highly readable.
//! - **Metadata Grouping**: Optional metadata (`CommitMeta`) is handled
//!   conditionally. If `meta` is provided, `Commit::with_meta` is used; otherwise,
//!   it falls back to `Commit::new`, which applies defaults.
//!
//! # Internal mechanism
//! The builder holds `Option` wrappers for required fields and a `Vec` for
//! parents. When [`build`] is called, it consumes the builder, extracts the
//! fields, and moves them directly into the new `Commit` struct without cloning.

use libvctrl_handler::{Commit, CommitMeta, Hash, UserID, VctrlError};

/// A builder for creating [`Commit`](libvctrl_handler::Commit) objects.
///
/// # Purpose
/// Provides a fluent interface for assembling a commit's data before finalizing
/// it into an immutable object.
///
/// # Design rationale
/// This struct derives [`Default`] so it can be easily instantiated, and
/// [`Debug`] for logging purposes. The `build` method consumes `self`,
/// preventing the reuse of the builder after the data has been moved.
///
/// # Examples
///
/// Building a standard commit:
///
/// ```
/// use libvctrl_core::object::CommitBuilder;
/// use libvctrl_handler::{Hash, UserID};
///
/// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let user = UserID::new("Alice".to_string(), "alice@example.com".to_string()).unwrap();
///
/// let commit = CommitBuilder::new()
///     .tree(tree)
///     .author(user.clone())
///     .committer(user)
///     .message("Initial commit")
///     .build()
///     .unwrap();
///
/// assert_eq!(commit.message(), "Initial commit");
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
    /// Creates a new, empty `CommitBuilder`.
    ///
    /// # Design rationale
    /// This is a `const fn`, allowing the builder to be instantiated in
    /// compile-time contexts if needed. All fields are initialized to `None`
    /// or empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::object::CommitBuilder;
    ///
    /// let builder = CommitBuilder::new();
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

    /// Sets the root tree hash for the commit.
    ///
    /// # Design rationale
    /// This is a required field. If `build` is called without setting this,
    /// it will fail. The method is `const fn` to maximize flexibility.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::object::CommitBuilder;
    /// use libvctrl_handler::Hash;
    ///
    /// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let builder = CommitBuilder::new().tree(tree);
    /// ```
    #[must_use]
    pub const fn tree(mut self, tree: Hash) -> Self {
        self.tree = Some(tree);
        self
    }

    /// Adds a parent commit hash.
    ///
    /// # Design rationale
    /// Commits can have zero or more parents (merge commits have multiple).
    /// This method appends to a vector, allowing it to be called multiple times.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::object::CommitBuilder;
    /// use libvctrl_handler::Hash;
    ///
    /// let parent1 = Hash::from_bytes(&[1u8; 64]).unwrap();
    /// let parent2 = Hash::from_bytes(&[2u8; 64]).unwrap();
    ///
    /// let builder = CommitBuilder::new()
    ///     .parent(parent1)
    ///     .parent(parent2);
    /// ```
    #[must_use]
    pub fn parent(mut self, parent: Hash) -> Self {
        self.parents.push(parent);
        self
    }

    /// Sets the author of the commit.
    ///
    /// # Design rationale
    /// This is a required field.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::object::CommitBuilder;
    /// use libvctrl_handler::UserID;
    ///
    /// let author = UserID::new("Alice".to_string(), "a@b.com".to_string()).unwrap();
    /// let builder = CommitBuilder::new().author(author);
    /// ```
    #[must_use]
    pub fn author(mut self, author: UserID) -> Self {
        self.author = Some(author);
        self
    }

    /// Sets the committer of the commit.
    ///
    /// # Design rationale
    /// This is a required field. In many cases, the committer is the same as
    /// the author, but the API enforces setting it explicitly.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::object::CommitBuilder;
    /// use libvctrl_handler::UserID;
    ///
    /// let committer = UserID::new("Bob".to_string(), "b@c.com".to_string()).unwrap();
    /// let builder = CommitBuilder::new().committer(committer);
    /// ```
    #[must_use]
    pub fn committer(mut self, committer: UserID) -> Self {
        self.committer = Some(committer);
        self
    }

    /// Sets the commit message.
    ///
    /// # Design rationale
    /// This is a required field. It takes `impl Into<String>` for ergonomics,
    /// allowing string literals (`&str`) or owned `String`s to be passed easily.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::object::CommitBuilder;
    ///
    /// let builder = CommitBuilder::new().message("Fix a bug");
    /// ```
    #[must_use]
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    /// Sets optional metadata (timestamp, timezone, encoding).
    ///
    /// # Design rationale
    /// If this method is not called, `build` will use default metadata (timestamp 0).
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::object::CommitBuilder;
    /// use libvctrl_handler::CommitMeta;
    ///
    /// let meta = CommitMeta { timestamp: 1000, ..Default::default() };
    /// let builder = CommitBuilder::new().meta(meta);
    /// ```
    #[must_use]
    pub fn meta(mut self, meta: CommitMeta) -> Self {
        self.meta = Some(meta);
        self
    }

    /// Consumes the builder and returns a finalized [`Commit`](libvctrl_handler::Commit).
    ///
    /// # Design rationale
    /// This method consumes `self` to enforce a linear flow. It validates that
    /// all required fields are present, returning a `Result` to gracefully
    /// handle missing data.
    ///
    /// # Errors
    /// Returns [`VctrlError::Other`](libvctrl_handler::VctrlError::Other) if
    /// `tree`, `author`, `committer`, or `message` have not been set.
    ///
    /// # Internal mechanism
    /// If `meta` was provided, it delegates to `Commit::with_meta`. Otherwise,
    /// it uses `Commit::new`.
    ///
    /// # Examples
    ///
    /// Successful build:
    ///
    /// ```
    /// use libvctrl_core::object::CommitBuilder;
    /// use libvctrl_handler::{Hash, UserID};
    ///
    /// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let user = UserID::new("A".to_string(), "a@a.com".to_string()).unwrap();
    ///
    /// let commit = CommitBuilder::new()
    ///     .tree(tree)
    ///     .author(user.clone())
    ///     .committer(user)
    ///     .message("msg")
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(commit.message(), "msg");
    /// ```
    ///
    /// Failed build (missing required fields):
    ///
    /// ```
    /// use libvctrl_core::object::CommitBuilder;
    /// use libvctrl_handler::VctrlError;
    ///
    /// let result = CommitBuilder::new().build();
    /// assert!(matches!(result, Err(VctrlError::Other(_))));
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
            Ok(Commit::with_meta(
                tree,
                self.parents,
                author,
                committer,
                message,
                meta,
            ))
        } else {
            Ok(Commit::new(tree, self.parents, author, committer, message))
        }
    }
}
