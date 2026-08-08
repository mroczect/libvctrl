//! Builder for [`Commit`] objects.

use libvctrl_handler::{Commit, Hash, UserID, VctrlError};

/// Builder for [`Commit`] objects.
///
/// All mandatory fields (tree, author, committer, message) must be set
/// before calling [`build`](CommitBuilder::build).
///
/// # Errors
/// Returns a [`VctrlError`] if any required field is missing.
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
