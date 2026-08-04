use crate::codec::Encoder;
use crate::domain::blob::Blob;
use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::domain::tree::{EntryKind, Tree, TreeEntry};
use crate::error::VctrlError;
use crate::hashing::Hasher;
use crate::merge::{ConflictResolver, ThreeWayMerge};
use crate::storage::traits::ObjectStore;
use std::collections::BTreeMap;

const MAX_DEPTH: usize = 1000;

pub struct ThreeWayMerger;

impl ThreeWayMerge for ThreeWayMerger {
    fn merge(
        &self,
        store: &mut dyn ObjectStore,
        base: &Hash,
        ours: &Hash,
        theirs: &Hash,
        resolver: &dyn ConflictResolver,
        encoder: &dyn Encoder,
        hasher: &dyn Hasher,
    ) -> Result<Hash, VctrlError> {
        self.merge_inner(store, base, ours, theirs, resolver, encoder, hasher, 0)
    }
}

impl ThreeWayMerger {
    #[allow(clippy::too_many_arguments)]
    fn merge_inner(
        &self,
        store: &mut dyn ObjectStore,
        base_hash: &Hash,
        ours_hash: &Hash,
        theirs_hash: &Hash,
        resolver: &dyn ConflictResolver,
        encoder: &dyn Encoder,
        hasher: &dyn Hasher,
        depth: usize,
    ) -> Result<Hash, VctrlError> {
        if depth > MAX_DEPTH {
            return Err(VctrlError::Other("max merge depth exceeded".into()));
        }
        let base_tree = get_tree(store, base_hash)?;
        let ours_tree = get_tree(store, ours_hash)?;
        let theirs_tree = get_tree(store, theirs_hash)?;
        let base_map = tree_to_map(&base_tree);
        let ours_map = tree_to_map(&ours_tree);
        let theirs_map = tree_to_map(&theirs_tree);

        let all_keys: Vec<String> = base_map
            .keys()
            .chain(ours_map.keys())
            .chain(theirs_map.keys())
            .cloned()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();

        let mut entries = Vec::new();
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
                        let base_data = get_blob(store, &b.hash)?;
                        let ours_data = get_blob(store, &o.hash)?;
                        let theirs_data = get_blob(store, &t.hash)?;
                        match resolver.resolve(&base_data, &ours_data, &theirs_data) {
                            Some(resolved) => {
                                let blob = Blob::new(resolved);
                                let blob_hash = hasher.hash_blob(blob.as_bytes());
                                store.put(&blob_hash, &Object::Blob(blob))?;
                                TreeEntry::new(key, EntryKind::Blob, blob_hash)
                            }
                            None => {
                                return Err(VctrlError::MergeConflict {
                                    entry: key,
                                    reason: "conflict".into(),
                                });
                            }
                        }
                    }
                    (EntryKind::Tree, EntryKind::Tree) => {
                        let merged = self.merge_inner(
                            store,
                            &b.hash,
                            &o.hash,
                            &t.hash,
                            resolver,
                            encoder,
                            hasher,
                            depth + 1,
                        )?;
                        TreeEntry::new(key, EntryKind::Tree, merged)
                    }
                    _ => {
                        return Err(VctrlError::MergeConflict {
                            entry: key,
                            reason: "type mismatch".into(),
                        });
                    }
                },
                _ => return Err(VctrlError::Other("unexpected merge state".into())),
            };
            entries.push(entry);
        }
        let new_tree = Tree::new(entries).map_err(VctrlError::Tree)?;
        let mut buf = Vec::new();
        encoder.encode_tree(&new_tree, &mut buf);
        let tree_hash = hasher.hash_tree_encoded(&buf);
        store.put(&tree_hash, &Object::Tree(new_tree))?;
        Ok(tree_hash)
    }
}

fn get_tree(store: &dyn ObjectStore, hash: &Hash) -> Result<Tree, VctrlError> {
    match store.get(hash)? {
        Some(Object::Tree(t)) => Ok(t),
        _ => Err(VctrlError::NotFound("tree".into())),
    }
}
fn get_blob(store: &dyn ObjectStore, hash: &Hash) -> Result<Vec<u8>, VctrlError> {
    match store.get(hash)? {
        Some(Object::Blob(b)) => Ok(b.into_bytes()),
        _ => Err(VctrlError::NotFound("blob".into())),
    }
}
fn tree_to_map(tree: &Tree) -> BTreeMap<String, TreeEntry> {
    let mut map = BTreeMap::new();
    for e in tree.entries() {
        map.insert(e.name.clone(), e.clone());
    }
    map
}
