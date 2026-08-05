use crate::command::Command;
use crate::domain::hash::Hash;
use crate::error::VctrlError;
use crate::storage::traits::{ObjectStore, ObjectStoreExt, RefStore};
use std::collections::HashSet;

const MAX_BLAME_ITERATIONS: usize = 100_000;

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
        if self.path.is_empty() {
            return Err(VctrlError::Other("empty path".into()));
        }

        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut current = Some(self.start_commit);
        let mut iterations = 0;

        let start_commit = store.get_commit(&self.start_commit)?;
        let start_tree = store.get_tree(&start_commit.tree)?;
        let first_blob = start_tree
            .entries()
            .iter()
            .find(|e| e.name == self.path)
            .ok_or_else(|| VctrlError::NotFound(format!("path '{}' not found", self.path)))?;
        let mut prev_blob_hash = first_blob.hash;
        let mut last_commit_hash = self.start_commit;
        let mut last_commit_info = (
            start_commit.author.clone(),
            start_commit.message.clone(),
            first_blob.hash,
        );

        while let Some(commit_hash) = current {
            iterations += 1;
            if iterations > MAX_BLAME_ITERATIONS {
                return Err(VctrlError::Other(
                    "blame: too many commits to process".into(),
                ));
            }

            if !visited.insert(commit_hash) {
                break;
            }
            let commit = store.get_commit(&commit_hash)?;
            let tree = store.get_tree(&commit.tree)?;

            let entry = match tree.entries().iter().find(|e| e.name == self.path) {
                Some(e) => e,
                None => {
                    break;
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

            last_commit_hash = commit_hash;
            last_commit_info = (commit.author.clone(), commit.message.clone(), entry.hash);

            current = commit.parents.first().copied();
        }

        if result.is_empty() || result.last().map(|e| e.commit_hash) != Some(last_commit_hash) {
            result.push(BlameEntry {
                commit_hash: last_commit_hash,
                blob_hash: last_commit_info.2,
                author: last_commit_info.0,
                message: last_commit_info.1,
            });
        }

        Ok(result)
    }
}
