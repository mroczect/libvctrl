use crate::codec::Encoder;
use crate::command::Command;
use crate::domain::commit::Commit;
use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::domain::user::UserInfo;
use crate::error::VctrlError;
use crate::hashing::Hasher;
use crate::storage::traits::{ObjectStore, RefStore};

pub struct CreateCommit {
    pub tree_hash: Hash,
    pub parents: Vec<Hash>,
    pub author: UserInfo,
    pub committer: UserInfo,
    pub message: String,
    pub encoder: Box<dyn Encoder>,
    pub hasher: Box<dyn Hasher>,
}
impl Command for CreateCommit {
    type Output = Hash;
    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<Hash, VctrlError> {
        let commit = Commit::new(
            self.tree_hash,
            self.parents.clone(),
            self.author.clone(),
            self.committer.clone(),
            self.message.clone(),
            None,
        );
        let mut buf = Vec::new();
        self.encoder.encode_commit(&commit, &mut buf);
        let hash = self.hasher.hash_commit_encoded(&buf);
        store.put(&hash, &Object::Commit(Box::new(commit)))?;
        if let Some(head_name) = get_head_branch_name(refs)? {
            refs.set_ref(&head_name, &hash)?;
        } else {
            refs.set_ref("refs/heads/main", &hash)?;
            refs.set_head("refs/heads/main")?;
        }
        Ok(hash)
    }
}
fn get_head_branch_name(refs: &dyn RefStore) -> Result<Option<String>, VctrlError> {
    if refs.head()?.is_some() {
        Ok(Some("refs/heads/main".to_string()))
    } else {
        Ok(None)
    }
}
