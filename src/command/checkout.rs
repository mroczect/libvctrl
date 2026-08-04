use crate::command::Command;
use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::domain::tree::{EntryKind, MAX_TREE_DEPTH};
use crate::error::VctrlError;
use crate::storage::traits::{ObjectStore, ObjectStoreExt, RefStore};

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
        let mut stack = vec![(self.tree_hash, String::new(), 0)];
        while let Some((hash, prefix, depth)) = stack.pop() {
            if depth > MAX_TREE_DEPTH {
                return Err(VctrlError::Other("max checkout depth exceeded".into()));
            }
            let tree = store.get_tree(&hash)?;
            for entry in tree.entries() {
                let path = if prefix.is_empty() {
                    entry.name.clone()
                } else {
                    let mut p = String::with_capacity(prefix.len() + 1 + entry.name.len());
                    p.push_str(&prefix);
                    p.push('/');
                    p.push_str(&entry.name);
                    p
                };
                match entry.kind {
                    EntryKind::Blob => match store.get(&entry.hash)? {
                        Some(Object::Blob(blob)) => files.push((path, blob.into_bytes())),
                        Some(_) => {
                            return Err(VctrlError::Other(format!(
                                "entry '{}' claims to be a blob but points to a different object type",
                                path
                            )));
                        }
                        None => {
                            return Err(VctrlError::NotFound(format!(
                                "blob for entry '{}' not found",
                                path
                            )));
                        }
                    },
                    EntryKind::Tree => {
                        stack.push((entry.hash, path, depth + 1));
                    }
                }
            }
        }
        Ok(files)
    }
}
