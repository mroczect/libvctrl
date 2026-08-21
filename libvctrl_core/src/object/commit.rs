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

#[cfg(test)]
mod tests {
    use super::*;

    fn hash_byte(byte: u8) -> Result<Hash, VctrlError> {
        Hash::from_bytes(&[byte; 64])
    }

    fn user(name: &str, email: &str) -> Result<UserID, VctrlError> {
        UserID::new(name.to_string(), email.to_string())
    }

    #[test]
    fn build_missing_tree_errors() -> Result<(), VctrlError> {
        let result = CommitBuilder::new()
            .author(user("A", "a@example.com")?)
            .committer(user("B", "b@example.com")?)
            .message("msg")
            .build();
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn build_missing_author_errors() -> Result<(), VctrlError> {
        let result = CommitBuilder::new()
            .tree(hash_byte(0x01)?)
            .committer(user("B", "b@example.com")?)
            .message("msg")
            .build();
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn build_missing_committer_errors() -> Result<(), VctrlError> {
        let result = CommitBuilder::new()
            .tree(hash_byte(0x01)?)
            .author(user("A", "a@example.com")?)
            .message("msg")
            .build();
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn build_missing_message_errors() -> Result<(), VctrlError> {
        let result = CommitBuilder::new()
            .tree(hash_byte(0x01)?)
            .author(user("A", "a@example.com")?)
            .committer(user("B", "b@example.com")?)
            .build();
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn build_valid_commit_without_meta() -> Result<(), VctrlError> {
        let tree = hash_byte(0x11)?;
        let parent = hash_byte(0x12)?;
        let author = user("Alice", "alice@example.com")?;
        let committer = user("Bob", "bob@example.com")?;
        let message = "hello".to_string();

        let commit = CommitBuilder::new()
            .tree(tree)
            .parent(parent)
            .author(author)
            .committer(committer)
            .message(message.clone())
            .build()?;

        assert_eq!(commit.tree(), &tree);
        assert_eq!(commit.parents(), &[parent]);
        assert_eq!(commit.author().name(), "Alice");
        assert_eq!(commit.committer().name(), "Bob");
        assert_eq!(commit.message(), message);
        Ok(())
    }

    #[test]
    fn build_valid_commit_with_meta() -> Result<(), VctrlError> {
        let tree = hash_byte(0x21)?;
        let author = user("Alice", "alice@example.com")?;
        let committer = user("Bob", "bob@example.com")?;
        let message = "hello".to_string();
        let meta = CommitMeta::new(123, 0, Some("utf-8".to_string()))?;

        let commit = CommitBuilder::new()
            .tree(tree)
            .author(author)
            .committer(committer)
            .message(message)
            .meta(meta)
            .build()?;

        assert_eq!(commit.meta().timestamp(), 123);
        assert_eq!(commit.meta().timezone_offset(), 0);
        assert_eq!(commit.meta().encoding(), Some("utf-8"));
        Ok(())
    }
}
