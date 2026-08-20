use libvctrl_handler::{Hash, ObjectStore, VctrlError};
use std::collections::HashMap;
use std::io::{Cursor, Read};

#[derive(Debug, Default)]
pub struct MemoryStore {
    objects: HashMap<Hash, Vec<u8>>,
}

impl MemoryStore {
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

    fn get(&self, hash: &Hash) -> Result<Box<dyn Read + Send + '_>, VctrlError> {
        let data = self
            .objects
            .get(hash)
            .ok_or(VctrlError::ObjectNotFound(*hash))?;
        Ok(Box::new(Cursor::new(data.as_slice())))
    }

    fn delete(&mut self, hash: &Hash) -> Result<(), VctrlError> {
        let _ = self.objects.remove(hash);
        Ok(())
    }

    fn exists(&self, hash: &Hash) -> Result<bool, VctrlError> {
        Ok(self.objects.contains_key(hash))
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
    fn test_put_and_get() {
        let mut store = MemoryStore::new();
        let hash = make_hash(0x01);
        let data = b"hello world";
        assert!(store.put(&hash, data).is_ok());

        let get_result = store.get(&hash);
        assert!(get_result.is_ok(), "should retrieve stored object");

        let mut reader = get_result.unwrap();
        let mut retrieved = Vec::new();
        Read::read_to_end(&mut reader, &mut retrieved).unwrap();
        assert_eq!(retrieved, data, "retrieved data must match original");
    }

    #[test]
    fn test_get_not_found() {
        let store = MemoryStore::new();
        let hash = make_hash(0xFF);
        let result = store.get(&hash);
        assert!(result.is_err(), "should error for missing object");
    }

    #[test]
    fn test_exists_false_then_true() {
        let mut store = MemoryStore::new();
        let hash = make_hash(0x10);
        assert_eq!(
            store.exists(&hash).unwrap(),
            false,
            "should not exist before put"
        );
        store.put(&hash, b"data").unwrap();
        assert_eq!(store.exists(&hash).unwrap(), true, "should exist after put");
    }

    #[test]
    fn test_delete_existing() {
        let mut store = MemoryStore::new();
        let hash = make_hash(0x20);
        store.put(&hash, b"to delete").unwrap();
        assert!(store.exists(&hash).unwrap());
        store.delete(&hash).unwrap();
        assert!(
            !store.exists(&hash).unwrap(),
            "should not exist after delete"
        );
    }

    #[test]
    fn test_delete_nonexistent() {
        let mut store = MemoryStore::new();
        let hash = make_hash(0x30);
        let result = store.delete(&hash);
        assert!(result.is_ok(), "deleting nonexistent key should not error");
    }

    #[test]
    fn test_overwrite() {
        let mut store = MemoryStore::new();
        let hash = make_hash(0x40);
        store.put(&hash, b"first version").unwrap();
        store.put(&hash, b"second version").unwrap();

        let mut reader = store.get(&hash).unwrap();
        let mut retrieved = Vec::new();
        Read::read_to_end(&mut reader, &mut retrieved).unwrap();
        assert_eq!(
            retrieved, b"second version",
            "should return the most recently put data"
        );
    }

    #[test]
    fn test_put_empty_data() {
        let mut store = MemoryStore::new();
        let hash = make_hash(0x50);
        store.put(&hash, b"").unwrap();
        let mut reader = store.get(&hash).unwrap();
        let mut retrieved = Vec::new();
        Read::read_to_end(&mut reader, &mut retrieved).unwrap();
        assert_eq!(retrieved, b"", "empty data should be stored and retrieved");
    }

    #[test]
    fn test_multiple_objects() {
        let mut store = MemoryStore::new();
        let h1 = make_hash(0x01);
        let h2 = make_hash(0x02);
        let h3 = make_hash(0x03);
        store.put(&h1, b"aaa").unwrap();
        store.put(&h2, b"bbb").unwrap();
        store.put(&h3, b"ccc").unwrap();

        assert!(store.exists(&h1).unwrap());
        assert!(store.exists(&h2).unwrap());
        assert!(store.exists(&h3).unwrap());

        store.delete(&h2).unwrap();
        assert!(store.exists(&h1).unwrap());
        assert!(!store.exists(&h2).unwrap());
        assert!(store.exists(&h3).unwrap());
    }
}
