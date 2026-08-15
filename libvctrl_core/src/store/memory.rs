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

    fn get(&self, hash: &Hash) -> Result<Box<dyn Read + '_>, VctrlError> {
        self.objects
            .get(hash)
            .cloned()
            .map(|v| Box::new(Cursor::new(v)) as Box<dyn Read>)
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

#[cfg(test)]
mod tests {
    use super::*;
    use libvctrl_handler::HASH_LENGTH;
    use std::io::Read;

    fn dummy_hash(byte: u8) -> Hash {
        let mut arr = [byte; HASH_LENGTH];
        arr[0] = byte;
        Hash::from_bytes(&arr).unwrap()
    }

    #[test]
    fn put_and_get() {
        let mut store = MemoryStore::new();
        let hash = dummy_hash(1);
        let data = b"hello world";
        store.put(&hash, data).unwrap();
        assert!(store.exists(&hash).unwrap());
        let mut buf = Vec::new();
        store.get(&hash).unwrap().read_to_end(&mut buf).unwrap();
        assert_eq!(buf, data);
    }

    #[test]
    fn get_non_existent_returns_error() {
        let store = MemoryStore::new();
        let hash = dummy_hash(2);
        assert!(store.get(&hash).is_err());
        assert!(!store.exists(&hash).unwrap());
    }

    #[test]
    fn delete_existing_object() {
        let mut store = MemoryStore::new();
        let hash = dummy_hash(3);
        store.put(&hash, b"data").unwrap();
        store.delete(&hash).unwrap();
        assert!(!store.exists(&hash).unwrap());
    }

    #[test]
    fn delete_non_existent_is_noop() {
        let mut store = MemoryStore::new();
        let hash = dummy_hash(4);
        assert!(store.delete(&hash).is_ok());
    }

    #[test]
    fn put_overwrites() {
        let mut store = MemoryStore::new();
        let hash = dummy_hash(5);
        store.put(&hash, b"old").unwrap();
        store.put(&hash, b"new").unwrap();
        let mut buf = Vec::new();
        store.get(&hash).unwrap().read_to_end(&mut buf).unwrap();
        assert_eq!(buf, b"new");
    }
}
