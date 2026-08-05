use crate::command::Command;
use crate::domain::hash::Hash;
use crate::error::VctrlError;
use crate::storage::traits::{ObjectStore, ObjectStoreExt, RefStore};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct BlameEntry {
    pub commit_hash: Hash,
    pub blob_hash: Hash,
    pub author: crate::domain::user::UserID,
    pub message: String,
}

pub struct Annotate {
    pub start_commit: Hash,
    pub path: String,
}

impl Command for Annotate {
    type Output = Vec<BlameEntry>;

    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        _refs: &mut dyn RefStore,
    ) -> Result<Vec<BlameEntry>, VctrlError> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut current = Some(self.start_commit);

        let start_commit = store.get_commit(&self.start_commit)?;
        let start_tree = store.get_tree(&start_commit.tree)?;
        let first_blob = start_tree
            .entries()
            .iter()
            .find(|e| e.name == self.path)
            .ok_or_else(|| VctrlError::NotFound(format!("path '{}' not found", self.path)))?;
        let mut prev_blob_hash = first_blob.hash;

        while let Some(commit_hash) = current {
            if !visited.insert(commit_hash) {
                break;
            }
            let commit = store.get_commit(&commit_hash)?;
            let tree = store.get_tree(&commit.tree)?;

            let entry = match tree.entries().iter().find(|e| e.name == self.path) {
                Some(e) => e,
                None => {
                    current = commit.parents.first().copied();
                    continue;
                }
            };

            if entry.hash != prev_blob_hash {
                result.push(BlameEntry {
                    commit_hash,
                    blob_hash: entry.hash,
                    author: commit.author.clone(),
                    message: commit.message.clone(),
                });
                prev_blob_hash = entry.hash;
            }

            current = commit.parents.first().copied();
        }
        Ok(result)
    }
}
