use crate::handler::error::VctrlError;
use crate::handler::types::{Blob, EntryKind, Hash, Object, ObjectStore, Tree, TreeEntry};
use std::collections::BTreeMap;

pub type MergeResolver = dyn Fn(&[u8], &[u8], &[u8]) -> Option<Vec<u8>>;

pub fn merge_trees(
    store: &mut dyn ObjectStore,
    base_hash: &Hash,
    ours_hash: &Hash,
    theirs_hash: &Hash,
    resolver: &MergeResolver,
) -> Result<Hash, VctrlError> {
    let base_tree = get_tree(store, base_hash)?;
    let ours_tree = get_tree(store, ours_hash)?;
    let theirs_tree = get_tree(store, theirs_hash)?;

    let base_map = tree_to_map(&base_tree);
    let ours_map = tree_to_map(&ours_tree);
    let theirs_map = tree_to_map(&theirs_tree);

    let mut all_keys: Vec<String> = base_map
        .keys()
        .chain(ours_map.keys())
        .chain(theirs_map.keys())
        .cloned()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    all_keys.sort();

    let mut new_entries = Vec::new();

    for key in all_keys {
        let base = base_map.get(&key);
        let ours = ours_map.get(&key);
        let theirs = theirs_map.get(&key);

        let entry = match (base, ours, theirs) {
            (None, Some(o), None) => o.clone(),
            (None, None, Some(t)) => t.clone(),
            (Some(_), None, Some(t)) => t.clone(),
            (Some(_), Some(o), None) => o.clone(),
            (Some(_), None, None) => continue,
            (Some(_), Some(o), Some(t)) if o.hash == t.hash => o.clone(),
            (Some(b), Some(o), Some(t)) if o.hash == b.hash => t.clone(),
            (Some(b), Some(o), Some(t)) if t.hash == b.hash => o.clone(),
            (Some(b), Some(o), Some(t)) => match (&o.kind, &t.kind) {
                (EntryKind::Blob, EntryKind::Blob) => {
                    let base_data = get_blob_bytes(store, &b.hash)?;
                    let ours_data = get_blob_bytes(store, &o.hash)?;
                    let theirs_data = get_blob_bytes(store, &t.hash)?;
                    if let Some(resolved) = resolver(&base_data, &ours_data, &theirs_data) {
                        let blob = Blob::new(resolved);
                        let blob_hash = store.put(&Object::Blob(blob))?;
                        TreeEntry::new(key, EntryKind::Blob, blob_hash)
                    } else {
                        return Err(VctrlError::MergeConflict {
                            entry: key,
                            reason: "resolver returned None".into(),
                        });
                    }
                }
                (EntryKind::Tree, EntryKind::Tree) => {
                    let merged_hash = merge_trees(store, &b.hash, &o.hash, &t.hash, resolver)?;
                    TreeEntry::new(key, EntryKind::Tree, merged_hash)
                }
                _ => {
                    return Err(VctrlError::MergeConflict {
                        entry: key,
                        reason: "type mismatch between ours and theirs".into(),
                    });
                }
            },
            _ => unreachable!(),
        };
        new_entries.push(entry);
    }

    let new_tree = Tree::new(new_entries).map_err(VctrlError::Tree)?;
    let tree_hash = store.put(&Object::Tree(new_tree))?;
    Ok(tree_hash)
}

fn get_tree(store: &dyn ObjectStore, hash: &Hash) -> Result<Tree, VctrlError> {
    match store.get(hash)? {
        Some(Object::Tree(t)) => Ok(t),
        _ => Err(VctrlError::NotFound("tree not found".into())),
    }
}

fn get_blob_bytes(store: &dyn ObjectStore, hash: &Hash) -> Result<Vec<u8>, VctrlError> {
    match store.get(hash)? {
        Some(Object::Blob(b)) => Ok(b.into_bytes()),
        _ => Err(VctrlError::NotFound("blob not found".into())),
    }
}

fn tree_to_map(tree: &Tree) -> BTreeMap<String, TreeEntry> {
    let mut map = BTreeMap::new();
    for entry in tree.entries() {
        map.insert(entry.name.clone(), entry.clone());
    }
    map
}
