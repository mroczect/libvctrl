use crate::codec::Encoder;
use crate::command::Command;
use crate::diff::{DiffEntry, DiffKind, TreeDiff, TreeDiffer};
use crate::domain::commit::Commit;
use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::domain::tree::{Tree, TreeEntry};
use crate::domain::user::UserID;
use crate::error::VctrlError;
use crate::hashing::Hasher;
use crate::storage::traits::{ObjectStore, RefStore};
use std::collections::BTreeMap;

pub struct Revert {
    pub commit_hash: Hash,
    pub author: UserID,
    pub committer: UserID,
    pub encoder: Box<dyn Encoder>,
    pub hasher: Box<dyn Hasher>,
}

impl Command for Revert {
    type Output = Hash;

    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<Hash, VctrlError> {
        let revert_commit = get_commit(store, &self.commit_hash)?;
        let revert_tree = get_tree(store, &revert_commit.tree)?;

        let parent_tree = match revert_commit.parents.first() {
            Some(parent_hash) => {
                let parent_commit = get_commit(store, parent_hash)?;
                get_tree(store, &parent_commit.tree)?
            }
            None => Tree::new(vec![]).map_err(VctrlError::Tree)?,
        };

        let differ = TreeDiffer;
        let diffs = differ.diff(&parent_tree, &revert_tree)?;

        let head_commit_hash = refs
            .head()?
            .ok_or_else(|| VctrlError::Other("no HEAD".into()))?;
        let head_commit = get_commit(store, &head_commit_hash)?;
        let head_tree = get_tree(store, &head_commit.tree)?;

        let new_tree = apply_reverse_diff(&head_tree, &parent_tree, &diffs)?;

        let mut buf = Vec::new();
        self.encoder.encode_tree(&new_tree, &mut buf);
        let new_tree_hash = self.hasher.hash_tree_encoded(&buf);
        store.put(&new_tree_hash, &Object::Tree(new_tree))?;

        let new_commit = Commit::new(
            new_tree_hash,
            vec![head_commit_hash],
            self.author.clone(),
            self.committer.clone(),
            format!("Revert: {}", &revert_commit.message),
            None,
        );
        let mut buf = Vec::new();
        self.encoder.encode_commit(&new_commit, &mut buf);
        let new_hash = self.hasher.hash_commit_encoded(&buf);
        store.put(&new_hash, &Object::Commit(Box::new(new_commit)))?;

        if let Some(branch_name) = refs.head_ref_name()? {
            refs.set_ref(&branch_name, &new_hash)?;
        }

        Ok(new_hash)
    }
}

fn get_commit(store: &dyn ObjectStore, hash: &Hash) -> Result<Commit, VctrlError> {
    match store.get(hash)? {
        Some(Object::Commit(c)) => Ok(*c),
        _ => Err(VctrlError::NotFound("commit not found".into())),
    }
}

fn get_tree(store: &dyn ObjectStore, hash: &Hash) -> Result<Tree, VctrlError> {
    match store.get(hash)? {
        Some(Object::Tree(t)) => Ok(t),
        _ => Err(VctrlError::NotFound("tree not found".into())),
    }
}

fn apply_reverse_diff(head: &Tree, parent: &Tree, diffs: &[DiffEntry]) -> Result<Tree, VctrlError> {
    let mut map: BTreeMap<String, TreeEntry> = head
        .entries()
        .iter()
        .map(|e| (e.name.clone(), e.clone()))
        .collect();

    let parent_map: BTreeMap<String, TreeEntry> = parent
        .entries()
        .iter()
        .map(|e| (e.name.clone(), e.clone()))
        .collect();

    for diff in diffs {
        match &diff.kind {
            DiffKind::Added => {
                map.remove(&diff.name);
            }
            DiffKind::Removed => {
                if map.contains_key(&diff.name) {
                    return Err(VctrlError::Other(format!(
                        "revert conflict: '{}' already exists in HEAD",
                        diff.name
                    )));
                }
                if let Some(entry) = parent_map.get(&diff.name) {
                    map.insert(diff.name.clone(), entry.clone());
                }
            }
            DiffKind::Modified { old_hash, .. } => {
                if let Some(current) = map.get(&diff.name)
                    && current.hash != *old_hash
                {
                    return Err(VctrlError::Other(format!(
                        "revert conflict: '{}' has been modified",
                        diff.name
                    )));
                }
                if let Some(entry) = map.get_mut(&diff.name) {
                    entry.hash = *old_hash;
                }
            }
        }
    }

    let entries: Vec<TreeEntry> = map.into_values().collect();
    Tree::new(entries).map_err(VctrlError::Tree)
}
