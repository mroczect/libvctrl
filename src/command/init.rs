use crate::codec::Encoder;
use crate::command::Command;
use crate::domain::commit::Commit;
use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::domain::tree::Tree;
use crate::domain::user::UserID;
use crate::error::VctrlError;
use crate::hashing::Hasher;
use crate::storage::traits::{ObjectStore, RefStore};

pub struct Init {
    pub author: UserID,
    pub encoder: Box<dyn Encoder>,
    pub hasher: Box<dyn Hasher>,
}

impl Command for Init {
    type Output = Hash;

    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<Hash, VctrlError> {
        let empty_tree = Tree::new(vec![]).map_err(VctrlError::Tree)?;
        let mut buf = Vec::new();
        self.encoder.encode_tree(&empty_tree, &mut buf)?;
        let tree_hash = self.hasher.hash_tree_encoded(&buf);
        store.put(&tree_hash, &Object::Tree(empty_tree))?;

        let commit = Commit::new(
            tree_hash,
            vec![],
            self.author.clone(),
            self.author.clone(),
            "initial commit".to_string(),
            None,
        );
        let mut buf = Vec::new();
        self.encoder.encode_commit(&commit, &mut buf)?;
        let commit_hash = self.hasher.hash_commit_encoded(&buf);
        store.put(&commit_hash, &Object::Commit(Box::new(commit)))?;

        refs.set_ref("refs/heads/main", &commit_hash)?;
        refs.set_head("refs/heads/main")?;

        Ok(commit_hash)
    }
}
