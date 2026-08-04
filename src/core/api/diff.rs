use crate::handler::error::VctrlError;
use crate::handler::types::{Hash, ObjectStore, Tree, TreeEntry};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub enum DiffKind {
    Added,
    Removed,
    Modified { old_hash: Hash, new_hash: Hash },
}

#[derive(Debug, Clone)]
pub struct DiffEntry {
    pub name: String,
    pub kind: DiffKind,
}

pub fn diff_trees(
    _store: &dyn ObjectStore,
    old_tree: &Tree,
    new_tree: &Tree,
) -> Result<Vec<DiffEntry>, VctrlError> {
    let old_map = tree_to_map(old_tree);
    let new_map = tree_to_map(new_tree);

    let mut keys: Vec<String> = old_map
        .keys()
        .chain(new_map.keys())
        .cloned()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    keys.sort();

    let mut diffs = Vec::new();

    for key in keys {
        let old = old_map.get(&key);
        let new = new_map.get(&key);

        match (old, new) {
            (None, Some(_)) => {
                diffs.push(DiffEntry {
                    name: key,
                    kind: DiffKind::Added,
                });
            }
            (Some(_), None) => {
                diffs.push(DiffEntry {
                    name: key,
                    kind: DiffKind::Removed,
                });
            }
            (Some(old_entry), Some(new_entry)) if old_entry.hash != new_entry.hash => {
                diffs.push(DiffEntry {
                    name: key,
                    kind: DiffKind::Modified {
                        old_hash: old_entry.hash,
                        new_hash: new_entry.hash,
                    },
                });
            }
            _ => {}
        }
    }

    Ok(diffs)
}

fn tree_to_map(tree: &Tree) -> BTreeMap<String, TreeEntry> {
    let mut map = BTreeMap::new();
    for entry in tree.entries() {
        map.insert(entry.name.clone(), entry.clone());
    }
    map
}
