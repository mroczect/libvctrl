use alloc::vec::IntoIter;
use std::collections::HashMap;

use libvctrl_handler::{Hash, RefStore, VctrlError};

#[derive(Debug, Default)]
pub struct MemoryRefStore {
    refs: HashMap<String, Hash>,
}

impl MemoryRefStore {
    #[must_use]
    pub fn new() -> Self {
        Self {
            refs: HashMap::new(),
        }
    }
}

impl RefStore for MemoryRefStore {
    type RefsIterator = IntoIter<Result<String, VctrlError>>;

    fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError> {
        libvctrl_handler::validate_ref_name(name)?;
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

    fn list_refs(&self) -> Result<Self::RefsIterator, VctrlError> {
        let mut names: Vec<String> = self.refs.keys().cloned().collect();
        names.sort();
        Ok(names.into_iter().map(Ok).collect::<Vec<_>>().into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash_byte(byte: u8) -> Result<Hash, VctrlError> {
        Hash::from_bytes(&[byte; 64])
    }

    #[test]
    fn set_and_get_ref_roundtrip() -> Result<(), VctrlError> {
        let mut store = MemoryRefStore::new();
        let hash = hash_byte(0xAB)?;

        store.set_ref("refs/heads/main", &hash)?;
        let got = store.get_ref("refs/heads/main")?;
        assert_eq!(got, hash);
        Ok(())
    }

    #[test]
    fn set_ref_invalid_name_errors() -> Result<(), VctrlError> {
        let mut store = MemoryRefStore::new();
        let hash = hash_byte(0xCD)?;
        assert!(store.set_ref("bad name", &hash).is_err());
        Ok(())
    }

    #[test]
    fn get_ref_missing_errors() {
        let store = MemoryRefStore::new();
        let result = store.get_ref("refs/heads/nope");
        assert!(matches!(result, Err(VctrlError::RefNotFound(_))));
    }

    #[test]
    fn delete_ref_removes_ref() -> Result<(), VctrlError> {
        let mut store = MemoryRefStore::new();
        let hash = hash_byte(0xEF)?;
        store.set_ref("refs/tags/v1", &hash)?;
        store.delete_ref("refs/tags/v1")?;
        assert!(store.get_ref("refs/tags/v1").is_err());
        Ok(())
    }

    #[test]
    fn list_refs_sorted() -> Result<(), VctrlError> {
        let mut store = MemoryRefStore::new();
        let h1 = hash_byte(0x01)?;
        let h2 = hash_byte(0x02)?;
        store.set_ref("refs/heads/b", &h1)?;
        store.set_ref("refs/heads/a", &h2)?;

        let names: Vec<String> = store.list_refs()?.collect::<Result<_, _>>()?;
        assert_eq!(
            names,
            vec!["refs/heads/a".to_string(), "refs/heads/b".to_string()]
        );
        Ok(())
    }
}
