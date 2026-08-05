use crate::domain::commit::Commit;
use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::error::VctrlError;
use crate::storage::traits::ObjectStore;
use std::collections::{BinaryHeap, HashSet};

#[derive(PartialEq, Eq)]
struct RevWalkItem {
    timestamp: chrono::DateTime<chrono::Utc>,
    hash: Hash,
}

impl Ord for RevWalkItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.timestamp
            .cmp(&other.timestamp)
            .then_with(|| self.hash.as_bytes().cmp(other.hash.as_bytes()))
    }
}
impl PartialOrd for RevWalkItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct RevWalk<'a> {
    store: &'a dyn ObjectStore,
    pending: BinaryHeap<RevWalkItem>,
    visited: HashSet<Hash>,
}

impl<'a> RevWalk<'a> {
    pub fn new(store: &'a dyn ObjectStore, tips: &[Hash]) -> Result<Self, VctrlError> {
        let mut walk = RevWalk {
            store,
            pending: BinaryHeap::new(),
            visited: HashSet::new(),
        };
        for tip in tips {
            walk.push(*tip)?;
        }
        Ok(walk)
    }

    fn push(&mut self, hash: Hash) -> Result<(), VctrlError> {
        if self.visited.contains(&hash) {
            return Ok(());
        }
        if let Some(Object::Commit(commit)) = self.store.get(&hash)? {
            self.visited.insert(hash);
            self.pending.push(RevWalkItem {
                timestamp: commit.timestamp,
                hash,
            });
        }
        Ok(())
    }
}

impl<'a> Iterator for RevWalk<'a> {
    type Item = Result<Commit, VctrlError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.pending.pop()?;
            let hash = item.hash;
            match self.store.get(&hash) {
                Ok(Some(Object::Commit(commit))) => {
                    for parent in &commit.parents {
                        if let Err(e) = self.push(*parent) {
                            return Some(Err(e));
                        }
                    }
                    return Some(Ok(*commit));
                }
                Ok(_) => continue,
                Err(e) => return Some(Err(e)),
            }
        }
    }
}
