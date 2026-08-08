//! Builder for [`Commit`] objects.

use libvctrl_handler::{Commit, Hash, UserID};

/// Builder for [`Commit`] objects.
///
/// All mandatory fields must be set before calling [`build`](CommitBuilder::build).
///
/// # Panics
/// Panics if a required field is missing. This is intentional: plumbing
/// code is expected to always set all fields before building.
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
    /// # Panics
    /// Panics if any mandatory field is missing.
    #[must_use]
    pub fn build(self) -> Commit {
        Commit::new(
            self.tree.expect("tree not set"),
            self.parents,
            self.author.expect("author not set"),
            self.committer.expect("committer not set"),
            self.message.expect("message not set"),
        )
    }
}
