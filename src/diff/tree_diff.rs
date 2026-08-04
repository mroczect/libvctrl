use crate::diff::{DiffEntry, DiffKind, TreeDiff};
use crate::domain::tree::{Tree, TreeEntry};
use crate::error::VctrlError;
use std::collections::BTreeMap;

pub struct TreeDiffer;
impl TreeDiff for TreeDiffer {
    fn diff(&self, old_tree: &Tree, new_tree: &Tree) -> Result<Vec<DiffEntry>, VctrlError> {
        let old_map = tree_to_map(old_tree);
        let new_map = tree_to_map(new_tree);
        let keys: std::collections::BTreeSet<String> =
            old_map.keys().chain(new_map.keys()).cloned().collect();
        let mut diffs = Vec::new();
        for key in keys {
            let old = old_map.get(&key);
            let new = new_map.get(&key);
            match (old, new) {
                (None, Some(_)) => diffs.push(DiffEntry {
                    name: key,
                    kind: DiffKind::Added,
                }),
                (Some(_), None) => diffs.push(DiffEntry {
                    name: key,
                    kind: DiffKind::Removed,
                }),
                (Some(o), Some(n)) if o.hash != n.hash => diffs.push(DiffEntry {
                    name: key,
                    kind: DiffKind::Modified {
                        old_hash: o.hash,
                        new_hash: n.hash,
                    },
                }),
                _ => {}
            }
        }
        Ok(diffs)
    }
}
fn tree_to_map(tree: &Tree) -> BTreeMap<String, TreeEntry> {
    let mut map = BTreeMap::new();
    for e in tree.entries() {
        map.insert(e.name.clone(), e.clone());
    }
    map
}
