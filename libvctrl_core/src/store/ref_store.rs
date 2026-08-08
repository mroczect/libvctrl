//! In‑memory reference store – the simplest [`RefStore`] implementation.

use libvctrl_handler::{Hash, MAX_NAME_LENGTH, RefStore, VctrlError};
use std::collections::HashMap;

/// An in‑memory reference store.
///
/// Validates names before storing them, as required by the trait contract.
///
/// # Characteristics
/// - **Not thread‑safe**: wrap in `Arc<Mutex<…>>` for shared access.
///
/// # Examples
/// ```
/// use libvctrl_core::store::MemoryRefStore;
/// use libvctrl_handler::{Hash, RefStore, HASH_LENGTH};
///
/// let mut refs = MemoryRefStore::new();
/// let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
/// refs.set_ref("HEAD", &hash).unwrap();
/// assert_eq!(refs.get_ref("HEAD").unwrap(), hash);
/// ```
#[derive(Debug, Default)]
pub struct MemoryRefStore {
    refs: HashMap<String, Hash>,
}

impl MemoryRefStore {
    /// Creates an empty reference store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            refs: HashMap::new(),
        }
    }
}

impl RefStore for MemoryRefStore {
    fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError> {
        if name.is_empty() || name.len() > MAX_NAME_LENGTH {
            return Err(VctrlError::InvalidName(name.into()));
        }
        let _ = self.refs.insert(name.to_string(), *hash);
        Ok(())
    }

    fn get_ref(&self, name: &str) -> Result<Hash, VctrlError> {
        self.refs
            .get(name)
            .copied()
            .ok_or_else(|| VctrlError::RefNotFound(name.into()))
    }

    fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError> {
        let _ = self.refs.remove(name);
        Ok(())
    }

    fn list_refs(&self) -> Result<Vec<String>, VctrlError> {
        Ok(self.refs.keys().cloned().collect())
    }
}
