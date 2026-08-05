use crate::codec::Encoder;
use crate::command::Command;
use crate::domain::commit::Commit;
use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::domain::user::UserID;
use crate::error::VctrlError;
use crate::hashing::Hasher;
use crate::merge::{ConflictResolver, ThreeWayMerge};
use crate::storage::traits::{ObjectStore, ObjectStoreExt, RefStore};
use std::collections::HashSet;

const MAX_REBASE_COMMITS: usize = 10_000;

pub struct Rebase {
    pub upstream: Hash,
    pub onto: Hash,
    pub author: UserID,
    pub committer: UserID,
    pub merger: Box<dyn ThreeWayMerge>,
    pub resolver: Box<dyn ConflictResolver>,
    pub encoder: Box<dyn Encoder>,
    pub hasher: Box<dyn Hasher>,
}

impl Command for Rebase {
    type Output = Hash;

    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<Hash, VctrlError> {
        let head_hash = refs
            .head()?
            .ok_or_else(|| VctrlError::Other("no HEAD".into()))?;

        let mut to_rebase = Vec::new();
        let mut visited = HashSet::new();
        let mut current = Some(head_hash);
        while let Some(h) = current {
            if h == self.upstream {
                break;
            }
            if !visited.insert(h) {
                return Err(VctrlError::Other("cycle detected".into()));
            }
            let commit = store.get_commit(&h)?;
            current = commit.parents.first().copied();
            to_rebase.push(commit);

            if to_rebase.len() > MAX_REBASE_COMMITS {
                return Err(VctrlError::Other(format!(
                    "too many commits to rebase (limit {})",
                    MAX_REBASE_COMMITS
                )));
            }
        }
        to_rebase.reverse();

        let mut current_head = self.onto;
        for src_commit in &to_rebase {
            let base_tree_hash = if let Some(parent_hash) = src_commit.parents.first() {
                let parent_commit = store.get_commit(parent_hash)?;
                parent_commit.tree
            } else {
                let empty_tree =
                    crate::domain::tree::Tree::new(vec![]).map_err(VctrlError::Tree)?;
                let mut buf = Vec::new();
                self.encoder.encode_tree(&empty_tree, &mut buf)?;
                let hash = self.hasher.hash_tree_encoded(&buf);
                store.put(&hash, &Object::Tree(empty_tree))?;
                hash
            };

            let merged_tree_hash = self.merger.merge(
                store,
                &base_tree_hash,
                &current_head,
                &src_commit.tree,
                self.resolver.as_ref(),
                self.encoder.as_ref(),
                self.hasher.as_ref(),
            )?;

            let new_commit = Commit::new(
                merged_tree_hash,
                vec![current_head],
                self.author.clone(),
                self.committer.clone(),
                src_commit.message.clone(),
                None,
            );
            let mut buf = Vec::new();
            self.encoder.encode_commit(&new_commit, &mut buf)?;
            let new_hash = self.hasher.hash_commit_encoded(&buf);
            store.put(&new_hash, &Object::Commit(Box::new(new_commit)))?;
            current_head = new_hash;
        }

        if let Some(head_ref) = refs.head_ref_name()? {
            refs.set_ref(&head_ref, &current_head)?;
        }
        Ok(current_head)
    }
}
