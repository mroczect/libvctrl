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
    use libvctrl_handler::HASH_LENGTH;

    fn make_hash(fill: u8) -> Hash {
        Hash::from_bytes(&vec![fill; HASH_LENGTH]).unwrap()
    }

    fn make_user_id(name: &str, email: &str) -> UserID {
        UserID::new(name.into(), email.into()).unwrap()
    }

    #[test]
    fn test_build_missing_tree() {
        let result = CommitBuilder::new()
            .author(make_user_id("A", "a@b.c"))
            .committer(make_user_id("B", "b@c.d"))
            .message("msg".into())
            .build();
        assert!(result.is_err(), "should fail without tree");
    }

    #[test]
    fn test_build_missing_author() {
        let result = CommitBuilder::new()
            .tree(make_hash(0))
            .committer(make_user_id("B", "b@c.d"))
            .message("msg".into())
            .build();
        assert!(result.is_err(), "should fail without author");
    }

    #[test]
    fn test_build_missing_committer() {
        let result = CommitBuilder::new()
            .tree(make_hash(0))
            .author(make_user_id("A", "a@b.c"))
            .message("msg".into())
            .build();
        assert!(result.is_err(), "should fail without committer");
    }

    #[test]
    fn test_build_missing_message() {
        let result = CommitBuilder::new()
            .tree(make_hash(0))
            .author(make_user_id("A", "a@b.c"))
            .committer(make_user_id("B", "b@c.d"))
            .build();
        assert!(result.is_err(), "should fail without message");
    }

    #[test]
    fn test_build_missing_all_required() {
        let result = CommitBuilder::new().build();
        assert!(result.is_err(), "should fail with no fields set");
    }

    #[test]
    fn test_build_success_without_meta() {
        let result = CommitBuilder::new()
            .tree(make_hash(1))
            .author(make_user_id("Alice", "alice@example.com"))
            .committer(make_user_id("Bob", "bob@example.com"))
            .message("initial commit".into())
            .build();
        assert!(result.is_ok(), "should succeed with all required fields");
    }

    #[test]
    fn test_build_success_with_meta() {
        let meta = CommitMeta::new(1700000000, 3600, Some("UTF-8".into())).unwrap();
        let result = CommitBuilder::new()
            .tree(make_hash(1))
            .author(make_user_id("Alice", "alice@example.com"))
            .committer(make_user_id("Bob", "bob@example.com"))
            .message("initial commit".into())
            .meta(meta)
            .build();
        assert!(result.is_ok(), "should succeed with meta");
    }

    #[test]
    fn test_build_with_multiple_parents() {
        let result = CommitBuilder::new()
            .tree(make_hash(1))
            .parent(make_hash(2))
            .parent(make_hash(3))
            .parent(make_hash(4))
            .author(make_user_id("Alice", "alice@example.com"))
            .committer(make_user_id("Bob", "bob@example.com"))
            .message("merge commit".into())
            .build();
        assert!(result.is_ok(), "should succeed with multiple parents");
        let commit = result.unwrap();
        assert_eq!(commit.parents().len(), 3);
    }

    #[test]
    fn test_build_with_meta_preserves_timestamp() {
        let meta = CommitMeta::new(9999999999, -7200, None).unwrap();
        let commit = CommitBuilder::new()
            .tree(make_hash(1))
            .author(make_user_id("A", "a@b.c"))
            .committer(make_user_id("B", "b@c.d"))
            .message("ts test".into())
            .meta(meta)
            .build()
            .unwrap();
        assert_eq!(commit.meta().timestamp(), 9999999999);
        assert_eq!(commit.meta().timezone_offset(), -7200);
    }
}
