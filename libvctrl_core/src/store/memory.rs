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

    fn hash_byte(byte: u8) -> Result<Hash, VctrlError> {
        Hash::from_bytes(&[byte; 64])
    }

    #[test]
    fn put_and_get_roundtrip() -> Result<(), VctrlError> {
        let mut store = MemoryStore::new();
        let hash = hash_byte(0xAB)?;
        let data = vec![10_u8, 20, 30];

        store.put(&hash, &data)?;
        {
            let mut reader = store.get(&hash)?;
            let mut buf = Vec::new();
            let _ = reader.read_to_end(&mut buf)?;
            assert_eq!(buf, data);
        }
        Ok(())
    }

    #[test]
    fn get_missing_object_errors() -> Result<(), VctrlError> {
        let store = MemoryStore::new();
        let hash = hash_byte(0xCD)?;
        let result = store.get(&hash);
        assert!(matches!(result, Err(VctrlError::ObjectNotFound(_))));
        Ok(())
    }

    #[test]
    fn delete_removes_object() -> Result<(), VctrlError> {
        let mut store = MemoryStore::new();
        let hash = hash_byte(0xEF)?;
        let data = vec![1_u8, 2, 3];

        store.put(&hash, &data)?;
        assert!(store.exists(&hash)?);
        store.delete(&hash)?;
        assert!(!store.exists(&hash)?);
        Ok(())
    }

    #[test]
    fn exists_missing_object_returns_false() -> Result<(), VctrlError> {
        let store = MemoryStore::new();
        let hash = hash_byte(0x77)?;
        assert!(!store.exists(&hash)?);
        Ok(())
    }
}
