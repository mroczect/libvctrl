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
}

impl Command for LogGraph {
    type Output = Vec<GraphCommit>;

    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        _refs: &mut dyn RefStore,
    ) -> Result<Vec<GraphCommit>, VctrlError> {
        let walk = RevWalk::new(store, &[self.head])?;
        let commit_pairs: Vec<(Hash, Commit)> = walk.collect::<Result<Vec<_>, _>>()?;

        let hash_to_idx: HashMap<Hash, usize> = commit_pairs
            .iter()
            .enumerate()
            .map(|(i, (hash, _))| (*hash, i))
            .collect();

        let graph_commits = commit_pairs
            .into_iter()
            .map(|(hash, c)| {
                let parent_indices = c
                    .parents
                    .iter()
                    .filter_map(|p| hash_to_idx.get(p).copied())
                    .collect();
                GraphCommit {
                    hash,
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
