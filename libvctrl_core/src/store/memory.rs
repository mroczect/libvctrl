//! In‑memory object store – the simplest [`ObjectStore`] implementation.

use libvctrl_handler::{Hash, ObjectStore, VctrlError};
use std::collections::HashMap;

/// An in‑memory object store backed by a [`HashMap`].
///
/// # Characteristics
/// - **Fast**: all operations are O(1) average.
/// - **Not thread‑safe**: wrap in `Arc<Mutex<…>>` for shared access.
/// - **Not persistent**: data is lost when the store is dropped.
///
/// # Examples
/// ```
/// use libvctrl_core::store::MemoryStore;
/// use libvctrl_handler::{Hash, ObjectStore, HASH_LENGTH};
///
/// let mut store = MemoryStore::new();
/// let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
/// store.put(&hash, b"hello").unwrap();
/// assert!(store.exists(&hash).unwrap());
/// assert_eq!(store.get(&hash).unwrap(), b"hello");
/// ```
#[derive(Debug, Default)]
pub struct MemoryStore {
    objects: HashMap<Hash, Vec<u8>>,
}

impl MemoryStore {
    /// Creates an empty store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
        }
    }
}

impl ObjectStore for MemoryStore {
    fn put(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError> {
        let _ = self.objects.insert(*hash, data.to_vec());
        Ok(())
    }

    fn get(&self, hash: &Hash) -> Result<Vec<u8>, VctrlError> {
        self.objects
            .get(hash)
            .cloned()
            .ok_or(VctrlError::ObjectNotFound(*hash))
    }

    fn delete(&mut self, hash: &Hash) -> Result<(), VctrlError> {
        let _ = self.objects.remove(hash);
        Ok(())
    }

    fn exists(&self, hash: &Hash) -> Result<bool, VctrlError> {
        Ok(self.objects.contains_key(hash))
    }
}
