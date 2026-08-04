use crate::handler::error::VctrlError;
use crate::handler::types::{EntryKind, Hash, Object, ObjectStore};

/// Rekursif mengekstrak semua blob dari tree menjadi daftar (path, konten).
/// Path adalah gabungan nama entry dengan separator `/`.
pub fn checkout_tree(
    store: &dyn ObjectStore,
    tree_hash: &Hash,
) -> Result<Vec<(String, Vec<u8>)>, VctrlError> {
    let mut files = Vec::new();
    checkout_recursive(store, tree_hash, "", &mut files)?;
    Ok(files)
}

fn checkout_recursive(
    store: &dyn ObjectStore,
    tree_hash: &Hash,
    prefix: &str,
    files: &mut Vec<(String, Vec<u8>)>,
) -> Result<(), VctrlError> {
    let tree = match store.get(tree_hash)? {
        Some(Object::Tree(t)) => t,
        _ => return Err(VctrlError::NotFound("tree not found".into())),
    };

    for entry in tree.entries() {
        let full_path = if prefix.is_empty() {
            entry.name.clone()
        } else {
            format!("{}/{}", prefix, entry.name)
        };

        match entry.kind {
            EntryKind::Blob => {
                let blob = match store.get(&entry.hash)? {
                    Some(Object::Blob(b)) => b,
                    _ => return Err(VctrlError::NotFound("blob not found".into())),
                };
                files.push((full_path, blob.into_bytes()));
            }
            EntryKind::Tree => {
                checkout_recursive(store, &entry.hash, &full_path, files)?;
            }
        }
    }
    Ok(())
}
