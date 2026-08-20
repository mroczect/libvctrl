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
    use libvctrl_handler::HASH_LENGTH;

    fn make_hash(fill: u8) -> Hash {
        Hash::from_bytes(&vec![fill; HASH_LENGTH]).unwrap()
    }

    #[test]
    fn test_set_and_get() {
        let mut store = MemoryRefStore::new();
        let hash = make_hash(0x01);
        assert!(store.set_ref("refs/heads/main", &hash).is_ok());

        let result = store.get_ref("refs/heads/main");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            hash,
            "retrieved hash must match stored hash"
        );
    }

    #[test]
    fn test_get_not_found() {
        let store = MemoryRefStore::new();
        let result = store.get_ref("refs/heads/nonexistent");
        assert!(result.is_err(), "should error for missing ref");
    }

    #[test]
    fn test_delete_existing() {
        let mut store = MemoryRefStore::new();
        let hash = make_hash(0x10);
        store.set_ref("refs/tags/v1", &hash).unwrap();
        assert!(store.get_ref("refs/tags/v1").is_ok());
        store.delete_ref("refs/tags/v1").unwrap();
        assert!(store.get_ref("refs/tags/v1").is_err());
    }

    #[test]
    fn test_delete_nonexistent() {
        let mut store = MemoryRefStore::new();
        let result = store.delete_ref("refs/heads/nope");
        assert!(result.is_ok(), "deleting nonexistent ref should not error");
    }

    #[test]
    fn test_list_refs_empty() {
        let store = MemoryRefStore::new();
        let refs: Vec<String> = store
            .list_refs()
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(refs.is_empty(), "new store should have no refs");
    }

    #[test]
    fn test_list_refs_sorted() {
        let mut store = MemoryRefStore::new();
        let h1 = make_hash(0x01);
        let h2 = make_hash(0x02);
        let h3 = make_hash(0x03);
        store.set_ref("refs/heads/main", &h1).unwrap();
        store.set_ref("refs/heads/feature", &h2).unwrap();
        store.set_ref("refs/tags/v1.0", &h3).unwrap();

        let refs: Vec<String> = store
            .list_refs()
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            refs,
            vec![
                "refs/heads/feature".to_string(),
                "refs/heads/main".to_string(),
                "refs/tags/v1.0".to_string(),
            ],
            "refs should be returned in sorted order"
        );
    }

    #[test]
    fn test_set_overwrite() {
        let mut store = MemoryRefStore::new();
        let h1 = make_hash(0xAA);
        let h2 = make_hash(0xBB);
        store.set_ref("refs/heads/main", &h1).unwrap();
        store.set_ref("refs/heads/main", &h2).unwrap();
        assert_eq!(
            store.get_ref("refs/heads/main").unwrap(),
            h2,
            "should return the most recently set hash"
        );
    }

    #[test]
    fn test_set_invalid_ref_name() {
        let mut store = MemoryRefStore::new();
        let hash = make_hash(0x00);
        let result = store.set_ref("invalid name with spaces", &hash);
        assert!(result.is_err(), "ref name with spaces should be rejected");
    }

    #[test]
    fn test_set_multiple_refs_independent() {
        let mut store = MemoryRefStore::new();
        let h_main = make_hash(0x01);
        let h_dev = make_hash(0x02);
        store.set_ref("refs/heads/main", &h_main).unwrap();
        store.set_ref("refs/heads/dev", &h_dev).unwrap();

        assert_eq!(store.get_ref("refs/heads/main").unwrap(), h_main);
        assert_eq!(store.get_ref("refs/heads/dev").unwrap(), h_dev);
        assert_eq!(
            store.list_refs().unwrap().count(),
            2,
            "should have exactly 2 refs"
        );
    }
}
