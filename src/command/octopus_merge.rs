use crate::codec::Encoder;
use crate::command::Command;
use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::domain::user::UserID;
use crate::error::VctrlError;
use crate::hashing::Hasher;
use crate::merge::{ConflictResolver, ThreeWayMerge, find_merge_base};
use crate::storage::traits::{ObjectStore, ObjectStoreExt, RefStore};

pub struct OctopusMerge {
    pub branch_names: Vec<String>,
    pub author: UserID,
    pub committer: UserID,
    pub merger: Box<dyn ThreeWayMerge>,
    pub resolver: Box<dyn ConflictResolver>,
    pub encoder: Box<dyn Encoder>,
    pub hasher: Box<dyn Hasher>,
}

impl Command for OctopusMerge {
    type Output = Hash;

    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<Hash, VctrlError> {
        if self.branch_names.len() < 2 {
            return Err(VctrlError::Other(
                "octopus merge requires at least 2 branches".into(),
            ));
        }

        let total_parents = 1 + self.branch_names.len();
        if total_parents > 255 {
            return Err(VctrlError::Other(format!(
                "too many parents: {} (max 255)",
                total_parents
            )));
        }

        let head_hash = refs
            .head()?
            .ok_or_else(|| VctrlError::Other("no HEAD".into()))?;
        let mut theirs_hashes = Vec::new();
        for name in &self.branch_names {
            theirs_hashes.push(
                refs.get_ref(name)?
                    .ok_or_else(|| VctrlError::NotFound(format!("branch '{}' not found", name)))?,
            );
        }

        let head_commit = store.get_commit(&head_hash)?;
        let mut current_tree = store.get_tree(&head_commit.tree)?;
        let mut parents = vec![head_hash];

        for theirs_hash in &theirs_hashes {
            let base = find_merge_base(store, head_hash, *theirs_hash)?
                .ok_or_else(|| VctrlError::Other("no common ancestor".into()))?;

            let mut buf = Vec::new();
            self.encoder.encode_tree(&current_tree, &mut buf)?;
            let our_tree_hash = self.hasher.hash_tree_encoded(&buf);
            if !store.exists(&our_tree_hash)? {
                store.put(&our_tree_hash, &Object::Tree(current_tree.clone()))?;
            }

            let merged_tree_hash = self.merger.merge(
                store,
                &base,
                &our_tree_hash,
                theirs_hash,
                self.resolver.as_ref(),
                self.encoder.as_ref(),
                self.hasher.as_ref(),
            )?;
            current_tree = store.get_tree(&merged_tree_hash)?;
            parents.push(*theirs_hash);
        }

        let mut buf = Vec::new();
        self.encoder.encode_tree(&current_tree, &mut buf)?;
        let final_tree_hash = self.hasher.hash_tree_encoded(&buf);
        if !store.exists(&final_tree_hash)? {
            store.put(&final_tree_hash, &Object::Tree(current_tree))?;
        }

        let commit = crate::domain::commit::Commit::new(
            final_tree_hash,
            parents,
            self.author.clone(),
            self.committer.clone(),
            format!("octopus merge of {}", self.branch_names.join(", ")),
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
