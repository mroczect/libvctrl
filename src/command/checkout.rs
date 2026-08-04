use crate::command::Command;
use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::domain::tree::EntryKind;
use crate::error::VctrlError;
use crate::storage::traits::{ObjectStore, RefStore};

const MAX_DEPTH: usize = 1000;

pub struct Checkout {
    pub tree_hash: Hash,
}

impl Command for Checkout {
    type Output = Vec<(String, Vec<u8>)>;

    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        _refs: &mut dyn RefStore,
    ) -> Result<Vec<(String, Vec<u8>)>, VctrlError> {
        let mut files = Vec::new();
        checkout_recursive(store, &self.tree_hash, "", &mut files, 0)?;
        Ok(files)
    }
}

fn checkout_recursive(
    store: &dyn ObjectStore,
    hash: &Hash,
    prefix: &str,
    files: &mut Vec<(String, Vec<u8>)>,
    depth: usize,
) -> Result<(), VctrlError> {
    if depth > MAX_DEPTH {
        return Err(VctrlError::Other("max checkout depth exceeded".into()));
    }
    match store.get(hash)? {
        Some(Object::Tree(tree)) => {
            for entry in tree.entries() {
                let path = if prefix.is_empty() {
                    entry.name.clone()
                } else {
                    let mut path = String::with_capacity(prefix.len() + 1 + entry.name.len());
                    path.push_str(prefix);
                    path.push('/');
                    path.push_str(&entry.name);
                    path
                };
                match entry.kind {
                    EntryKind::Blob => {
                        if let Some(Object::Blob(blob)) = store.get(&entry.hash)? {
                            files.push((path, blob.into_bytes()));
                        }
                    }
                    EntryKind::Tree => {
                        checkout_recursive(store, &entry.hash, &path, files, depth + 1)?;
                    }
                }
            }
        }
        _ => return Err(VctrlError::NotFound("tree not found".into())),
    }
    Ok(())
}
