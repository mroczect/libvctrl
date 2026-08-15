use libvctrl_handler::{Commit, CommitMeta, Hash, UserID, VctrlError};


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

    
    #[must_use]
    pub const fn tree(mut self, tree: Hash) -> Self {
        self.tree = Some(tree);
        self
    }

    
    #[must_use]
    pub fn parent(mut self, parent: Hash) -> Self {
        self.parents.push(parent);
        self
    }

    
    #[must_use]
    pub fn author(mut self, author: UserID) -> Self {
        self.author = Some(author);
        self
    }

    
    #[must_use]
    pub fn committer(mut self, committer: UserID) -> Self {
        self.committer = Some(committer);
        self
    }

    
    #[must_use]
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    
    #[must_use]
    pub fn meta(mut self, meta: CommitMeta) -> Self {
        self.meta = Some(meta);
        self
    }

    
    
    
    
    
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
