use libvctrl_handler::{Hash, RefStore, VctrlError};
use std::collections::HashMap;

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
    type RefsIterator = std::vec::IntoIter<Result<String, VctrlError>>;

    fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError> {
        if name.is_empty()
            || name.len()
                > usize::try_from(libvctrl_handler::MAX_NAME_LENGTH)
                    .expect("MAX_NAME_LENGTH too large")
        {
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

    fn dummy_hash() -> Hash {
        Hash::from_bytes(&[0xAB; HASH_LENGTH]).unwrap()
    }

    #[test]
    fn set_and_get_ref() {
        let mut store = MemoryRefStore::new();
        let hash = dummy_hash();
        store.set_ref("HEAD", &hash).unwrap();
        assert_eq!(store.get_ref("HEAD").unwrap(), hash);
    }

    #[test]
    fn get_non_existent_ref() {
        let store = MemoryRefStore::new();
        assert!(store.get_ref("HEAD").is_err());
    }

    #[test]
    fn delete_ref() {
        let mut store = MemoryRefStore::new();
        store.set_ref("refs/heads/main", &dummy_hash()).unwrap();
        store.delete_ref("refs/heads/main").unwrap();
        assert!(store.get_ref("refs/heads/main").is_err());
    }

    #[test]
    fn delete_non_existent_is_noop() {
        let mut store = MemoryRefStore::new();
        assert!(store.delete_ref("nope").is_ok());
    }

    #[test]
    fn set_ref_with_empty_name_fails() {
        let mut store = MemoryRefStore::new();
        assert!(store.set_ref("", &dummy_hash()).is_err());
    }

    #[test]
    fn set_ref_with_too_long_name_fails() {
        let mut store = MemoryRefStore::new();
        let long_name = "a".repeat(
            usize::try_from(libvctrl_handler::MAX_NAME_LENGTH).expect("MAX_NAME_LENGTH too large")
                + 1,
        );
        assert!(store.set_ref(&long_name, &dummy_hash()).is_err());
    }

    #[test]
    fn list_refs() {
        let mut store = MemoryRefStore::new();
        store.set_ref("a", &dummy_hash()).unwrap();
        store.set_ref("b", &dummy_hash()).unwrap();
        let iter = store.list_refs().unwrap();
        let list: Vec<String> = iter.collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&"a".to_string()));
        assert!(list.contains(&"b".to_string()));
    }

    #[test]
    fn overwrite_ref() {
        let mut store = MemoryRefStore::new();
        let hash1 = dummy_hash();
        let mut hash2_arr = [0xCD; HASH_LENGTH];
        hash2_arr[0] = 0xCD;
        let hash2 = Hash::from_bytes(&hash2_arr).unwrap();
        store.set_ref("HEAD", &hash1).unwrap();
        store.set_ref("HEAD", &hash2).unwrap();
        assert_eq!(store.get_ref("HEAD").unwrap(), hash2);
    }
}
