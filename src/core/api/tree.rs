use crate::handler::error::VctrlError;
use crate::handler::types::{EntryKind, Hash, Object, ObjectStore};

pub fn checkout_tree(
    store: &dyn ObjectStore,
    tree_hash: &Hash,
) -> Result<Vec<(String, Vec<u8>)>, VctrlError> {
    let tree = match store.get(tree_hash)? {
        Some(Object::Tree(t)) => t,
        _ => return Err(VctrlError::NotFound("tree not found".into())),
    };

    let mut files = Vec::new();
    for entry in tree.entries() {
        if entry.kind == EntryKind::Blob {
            let blob = match store.get(&entry.hash)? {
                Some(Object::Blob(b)) => b,
                _ => return Err(VctrlError::NotFound("blob not found".into())),
            };
            files.push((entry.name.clone(), blob.into_bytes()));
        }
    }
    Ok(files)
}
