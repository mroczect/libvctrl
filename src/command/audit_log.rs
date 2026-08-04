use crate::command::Command;
use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::error::VctrlError;
use crate::storage::traits::{ObjectStore, RefStore};
use chrono::{DateTime, Utc};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub commit_hash: Hash,
    pub tree_hash: Hash,
    pub parents: Vec<Hash>,
    pub author: String,
    pub committer: String,
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub has_signature: bool,
}

pub struct AuditLog;

impl Command for AuditLog {
    type Output = Vec<AuditEntry>;

    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<Vec<AuditEntry>, VctrlError> {
        let head = match refs.head()? {
            Some(h) => h,
            None => return Ok(Vec::new()),
        };
        let mut entries = Vec::with_capacity(16);
        let mut visited = HashSet::new();
        let mut current = Some(head);
        while let Some(hash) = current {
            if !visited.insert(hash) {
                return Err(VctrlError::Other("commit graph cycle detected".into()));
            }
            match store.get(&hash)? {
                Some(Object::Commit(c)) => {
                    let commit = *c;
                    entries.push(AuditEntry {
                        commit_hash: hash,
                        tree_hash: commit.tree,
                        parents: commit.parents.clone(),
                        author: format!("{} <{}>", commit.author.name, commit.author.email),
                        committer: format!(
                            "{} <{}>",
                            commit.committer.name, commit.committer.email
                        ),
                        timestamp: commit.timestamp,
                        message: commit.message.clone(),
                        has_signature: commit.signature.is_some(),
                    });
                    current = commit.parents.first().copied();
                }
                _ => break,
            }
        }
        Ok(entries)
    }
}
