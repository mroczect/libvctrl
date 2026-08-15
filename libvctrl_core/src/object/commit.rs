use libvctrl_handler::{Commit, CommitMeta, Hash, UserID, VctrlError};

/// A builder for creating `Commit` objects.
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
    /// Creates a new `CommitBuilder`.
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

    /// Sets the tree hash.
    #[must_use]
    pub const fn tree(mut self, tree: Hash) -> Self {
        self.tree = Some(tree);
        self
    }

    /// Adds a parent commit hash.
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

    /// Sets the commit metadata.
    #[must_use]
    pub fn meta(mut self, meta: CommitMeta) -> Self {
        self.meta = Some(meta);
        self
    }

    /// Builds the `Commit`.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if validation fails.
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
