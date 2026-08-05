use crate::codec::Encoder;
use crate::command::Command;
use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::domain::user::UserID;
use crate::error::VctrlError;
use crate::hashing::Hasher;
use crate::merge::{ConflictResolver, ThreeWayMerge, find_merge_base, is_ancestor};
use crate::storage::traits::{ObjectStore, RefStore};

pub struct MergeBranch {
    pub branch_name: String,
    pub author: UserID,
    pub committer: UserID,
    pub merger: Box<dyn ThreeWayMerge>,
    pub resolver: Box<dyn ConflictResolver>,
    pub encoder: Box<dyn Encoder>,
    pub hasher: Box<dyn Hasher>,
}

impl Command for MergeBranch {
    type Output = Hash;

    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<Hash, VctrlError> {
        let head_hash = refs
            .head()?
            .ok_or_else(|| VctrlError::Other("no HEAD".into()))?;
        let theirs_hash = refs.get_ref(&self.branch_name)?.ok_or_else(|| {
            VctrlError::NotFound(format!("branch '{}' not found", self.branch_name))
        })?;

        if is_ancestor(store as &dyn ObjectStore, head_hash, theirs_hash)? {
            if let Some(head_ref) = refs.head_ref_name()? {
                refs.set_ref(&head_ref, &theirs_hash)?;
            }
            return Ok(theirs_hash);
        }

        let base = find_merge_base(store, head_hash, theirs_hash)?
            .ok_or_else(|| VctrlError::Other("no common ancestor".into()))?;

        let merged_tree = self.merger.merge(
            store,
            &base,
            &head_hash,
            &theirs_hash,
            self.resolver.as_ref(),
            self.encoder.as_ref(),
            self.hasher.as_ref(),
        )?;

        let commit = crate::domain::commit::Commit::new(
            merged_tree,
            vec![head_hash, theirs_hash],
            self.author.clone(),
            self.committer.clone(),
            format!("merge branch '{}'", self.branch_name),
            None,
        );

        let mut buf = Vec::new();
        self.encoder.encode_commit(&commit, &mut buf)?;
        let commit_hash = self.hasher.hash_commit_encoded(&buf);
        store.put(&commit_hash, &Object::Commit(Box::new(commit)))?;

        if let Some(head_ref) = refs.head_ref_name()? {
            refs.set_ref(&head_ref, &commit_hash)?;
        }

        Ok(commit_hash)
    }
}
