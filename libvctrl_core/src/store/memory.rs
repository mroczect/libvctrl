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
