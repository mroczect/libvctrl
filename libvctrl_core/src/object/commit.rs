//! Builder for [`Commit`] objects.
//!
//! A [`CommitBuilder`] constructs a [`Commit`] step by step. All mandatory
//! fields (`tree`, `author`, `committer`, `message`) must be set before
//! calling [`build`](CommitBuilder::build). If any required field is missing,
//! an error is returned.
//!
//! # Why use a builder?
//!
//! Commits can have many parents, and the author and committer may be
//! different. The builder pattern makes it clear which field is which,
//! especially when method chaining:
//!
//! ```rust
//! # use libvctrl_core::object::CommitBuilder;
//! # use libvctrl_handler::*;
//! let hash = Hash::from_bytes(&[0xAB; 64]).unwrap();
//! let user = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
//!
//! let commit = CommitBuilder::new()
//!     .tree(hash)
//!     .parent(hash)          // first parent
//!     .parent(hash)          // second parent
//!     .author(user.clone())
//!     .committer(user)
//!     .message("Merge branch 'feature'")
//!     .build()
//!     .unwrap();
//! ```
//!
//! # Error handling
//!
//! The `build()` method returns `Result<Commit, VctrlError>`. It will
//! fail with `VctrlError::Other` if any of the required fields is missing.
//! This is a deliberate design choice: instead of panicking, we give the
//! caller a chance to handle the error gracefully.
//!
//! Note that the builder does **not** validate the content of hashes or
//! user identities – those are already guaranteed to be valid by their
//! own constructors in `libvctrl_handler`.

use libvctrl_handler::{Commit, Hash, UserID, VctrlError};

/// Builder for [`Commit`] objects.
///
/// All mandatory fields (tree, author, committer, message) must be set
/// before calling [`build`](CommitBuilder::build).
///
/// # Errors
/// Returns a [`VctrlError`] if any required field is missing.
///
/// # Example
///
/// ```rust
/// # use libvctrl_core::object::CommitBuilder;
/// # use libvctrl_handler::*;
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let user = UserID::new("A".into(), "a@b.c".into()).unwrap();
/// let commit = CommitBuilder::new()
///     .tree(hash)
///     .author(user.clone())
///     .committer(user)
///     .message("msg")
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Default)]
pub struct CommitBuilder {
    tree: Option<Hash>,
    parents: Vec<Hash>,
    author: Option<UserID>,
    committer: Option<UserID>,
    message: Option<String>,
}

impl CommitBuilder {
    /// Creates a new empty builder.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            tree: None,
            parents: Vec::new(),
            author: None,
            committer: None,
            message: None,
        }
    }

    /// Sets the tree hash.
    #[must_use]
    pub const fn tree(mut self, tree: Hash) -> Self {
        self.tree = Some(tree);
        self
    }

    /// Adds a parent hash.
    #[must_use]
    pub fn parent(mut self, parent: Hash) -> Self {
        self.parents.push(parent);
        self
    }

    /// Sets the author.
    #[must_use]
    pub fn author(mut self, author: UserID) -> Self {
        self.author = Some(author);
        self
    }

    /// Sets the committer.
    #[must_use]
    pub fn committer(mut self, committer: UserID) -> Self {
        self.committer = Some(committer);
        self
    }

    /// Sets the commit message.
    #[must_use]
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    /// Builds the commit.
    ///
    /// # Errors
    /// Returns `VctrlError::Other` if any mandatory field is missing.
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
        Ok(Commit::new(tree, self.parents, author, committer, message))
    }
}
