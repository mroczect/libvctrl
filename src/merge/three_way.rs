use crate::codec::Encoder;
use crate::domain::blob::Blob;
use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::domain::tree::{EntryKind, MAX_TREE_DEPTH, Tree, TreeEntry};
use crate::error::VctrlError;
use crate::hashing::Hasher;
use crate::merge::{ConflictResolver, ThreeWayMerge};
use crate::storage::traits::{ObjectStore, ObjectStoreExt};
use std::collections::BTreeMap;

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
        if depth > MAX_TREE_DEPTH {
            return Err(VctrlError::Other("max merge depth exceeded".into()));
        }
        let base_tree = store.get_tree(base_hash)?;
        let ours_tree = store.get_tree(ours_hash)?;
        let theirs_tree = store.get_tree(theirs_hash)?;
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
                        let base_data = store.get_blob(&b.hash)?;
                        let ours_data = store.get_blob(&o.hash)?;
                        let theirs_data = store.get_blob(&t.hash)?;
                        match resolver.resolve(&base_data, &ours_data, &theirs_data) {
                            Some(resolved) => {
                                let blob = Blob::new(resolved);
                                let blob_hash = hasher.hash_blob(blob.as_bytes());
                                store.put(&blob_hash, &Object::Blob(blob))?;
                                TreeEntry::new(key, EntryKind::Blob, blob_hash)
                                    .map_err(VctrlError::Tree)?
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
                        TreeEntry::new(key, EntryKind::Tree, merged).map_err(VctrlError::Tree)?
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

fn tree_to_map(tree: &Tree) -> BTreeMap<String, TreeEntry> {
    let mut map = BTreeMap::new();
    for e in tree.entries() {
        map.insert(e.name.clone(), e.clone());
    }
    map
}
