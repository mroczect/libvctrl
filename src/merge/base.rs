use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::error::VctrlError;
use crate::storage::traits::ObjectStore;
use std::collections::{HashSet, VecDeque};

pub fn find_merge_base(
    store: &dyn ObjectStore,
    a: Hash,
    b: Hash,
) -> Result<Option<Hash>, VctrlError> {
    let mut ancestors = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(a);
    while let Some(hash) = queue.pop_front() {
        if !ancestors.insert(hash) {
            continue;
        }
        if let Some(Object::Commit(commit)) = store.get(&hash)? {
            for parent in &commit.parents {
                queue.push_back(*parent);
            }
        }
    }

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(b);
    while let Some(hash) = queue.pop_front() {
        if !visited.insert(hash) {
            continue;
        }
        if ancestors.contains(&hash) {
            return Ok(Some(hash));
        }
        if let Some(Object::Commit(commit)) = store.get(&hash)? {
            for parent in &commit.parents {
                queue.push_back(*parent);
            }
        }
    }
    Ok(None)
}
