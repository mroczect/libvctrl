use crate::command::Command;
use crate::domain::commit::Commit;
use crate::domain::hash::Hash;
use crate::domain::user::UserID;
use crate::error::VctrlError;
use crate::revwalk::RevWalk;
use crate::storage::traits::{ObjectStore, RefStore};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct GraphCommit {
    pub hash: Hash,
    pub message: String,
    pub author: UserID,
    pub timestamp: DateTime<Utc>,
    pub parent_indices: Vec<usize>,
}

pub struct LogGraph {
    pub head: Hash,
    pub encoder: Box<dyn crate::codec::Encoder>,
    pub hasher: Box<dyn crate::hashing::Hasher>,
}

impl Command for LogGraph {
    type Output = Vec<GraphCommit>;

    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        _refs: &mut dyn RefStore,
    ) -> Result<Vec<GraphCommit>, VctrlError> {
        let walk = RevWalk::new(store, &[self.head])?;
        let commits: Vec<Commit> = walk.collect::<Result<Vec<_>, _>>()?;

        let mut hashes = Vec::with_capacity(commits.len());
        for c in &commits {
            let mut buf = Vec::new();
            self.encoder.encode_commit(c, &mut buf)?;
            let hash = self.hasher.hash_commit_encoded(&buf);
            hashes.push(hash);
        }

        let hash_to_idx: HashMap<Hash, usize> =
            hashes.iter().enumerate().map(|(i, h)| (*h, i)).collect();

        let graph_commits = commits
            .into_iter()
            .enumerate()
            .map(|(i, c)| {
                let parent_indices = c
                    .parents
                    .iter()
                    .filter_map(|p| hash_to_idx.get(p).copied())
                    .collect();
                GraphCommit {
                    hash: hashes[i],
                    message: c.message,
                    author: c.author,
                    timestamp: c.timestamp,
                    parent_indices,
                }
            })
            .collect();

        Ok(graph_commits)
    }
}
