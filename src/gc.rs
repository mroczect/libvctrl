use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::error::VctrlError;
use crate::storage::traits::{ObjectStore, RefStore};
use std::collections::HashSet;

pub fn mark_reachable(
    store: &dyn ObjectStore,
    refs: &dyn RefStore,
) -> Result<HashSet<Hash>, VctrlError> {
    let mut reachable = HashSet::new();
    let mut to_visit: Vec<Hash> = Vec::new();

    for ref_name in refs.list_refs("refs/")? {
        if let Some(hash) = refs.get_ref(&ref_name)? {
            to_visit.push(hash);
        }
    }
    if let Some(head) = refs.head()? {
        to_visit.push(head);
    }

    while let Some(hash) = to_visit.pop() {
        if !reachable.insert(hash) {
            continue;
        }
        if let Some(obj) = store.get(&hash)? {
            match obj {
                Object::Commit(c) => {
                    to_visit.push(c.tree);
                    for p in &c.parents {
                        to_visit.push(*p);
                    }
                }
                Object::Tree(t) => {
                    for entry in t.entries() {
                        to_visit.push(entry.hash);
                    }
                }
                Object::Tag(t) => {
                    to_visit.push(t.target);
                }
                _ => {}
            }
        }
    }
    Ok(reachable)
}

pub fn gc(store: &mut dyn ObjectStore, refs: &dyn RefStore) -> Result<usize, VctrlError> {
    let reachable = mark_reachable(store, refs)?;
    Ok(reachable.len())
}
