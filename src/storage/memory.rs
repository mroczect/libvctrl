use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::error::VctrlError;
use crate::reflog::ReflogEntry;
use crate::storage::traits::{ObjectStore, RefStore};
use std::collections::HashMap;

pub struct MemoryStore {
    objects: HashMap<Hash, Object>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
        }
    }
}
impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}
impl ObjectStore for MemoryStore {
    fn put(&mut self, hash: &Hash, obj: &Object) -> Result<(), VctrlError> {
        self.objects.insert(*hash, obj.clone());
        Ok(())
    }
    fn get(&self, hash: &Hash) -> Result<Option<Object>, VctrlError> {
        Ok(self.objects.get(hash).cloned())
    }
    fn exists(&self, hash: &Hash) -> Result<bool, VctrlError> {
        Ok(self.objects.contains_key(hash))
    }
    fn all_hashes(&self) -> Result<Vec<Hash>, VctrlError> {
        Ok(self.objects.keys().copied().collect())
    }
    fn remove(&mut self, hash: &Hash) -> Result<(), VctrlError> {
        self.objects.remove(hash);
        Ok(())
    }
}

pub struct MemoryRefStore {
    refs: HashMap<String, Hash>,
    head: Option<String>,
    reflog: Vec<ReflogEntry>,
}

impl MemoryRefStore {
    pub fn new() -> Self {
        Self {
            refs: HashMap::new(),
            head: None,
            reflog: Vec::new(),
        }
    }
}
impl Default for MemoryRefStore {
    fn default() -> Self {
        Self::new()
    }
}

impl RefStore for MemoryRefStore {
    fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError> {
        let old = self.refs.get(name).copied();
        self.refs.insert(name.to_string(), *hash);
        self.reflog.push(ReflogEntry {
            ref_name: name.to_string(),
            old_hash: old,
            new_hash: *hash,
            timestamp: chrono::Utc::now(),
            message: format!("set_ref {}", name),
        });
        Ok(())
    }
    fn get_ref(&self, name: &str) -> Result<Option<Hash>, VctrlError> {
        Ok(self.refs.get(name).copied())
    }
    fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError> {
        self.refs.remove(name);
        Ok(())
    }
    fn set_head(&mut self, target: &str) -> Result<(), VctrlError> {
        self.head = Some(target.to_string());
        Ok(())
    }
    fn head(&self) -> Result<Option<Hash>, VctrlError> {
        match &self.head {
            Some(target) if target.starts_with("refs/") => self.get_ref(target),
            Some(direct) => Hash::from_hex(direct).map(Some).map_err(VctrlError::Hash),
            None => Ok(None),
        }
    }
    fn head_ref_name(&self) -> Result<Option<String>, VctrlError> {
        match &self.head {
            Some(target) if target.starts_with("refs/heads/") => {
                if self.refs.contains_key(target) {
                    Ok(Some(target.clone()))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
    fn list_refs(&self, prefix: &str) -> Result<Vec<String>, VctrlError> {
        Ok(self
            .refs
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect())
    }
}

impl crate::reflog::ReflogStore for MemoryRefStore {
    fn log_ref_update(
        &mut self,
        ref_name: &str,
        old_hash: Option<Hash>,
        new_hash: Hash,
        message: &str,
    ) -> Result<(), VctrlError> {
        self.reflog.push(ReflogEntry {
            ref_name: ref_name.to_string(),
            old_hash,
            new_hash,
            timestamp: chrono::Utc::now(),
            message: message.to_string(),
        });
        Ok(())
    }
    fn reflog(&self, ref_name: &str) -> Result<Vec<ReflogEntry>, VctrlError> {
        Ok(self
            .reflog
            .iter()
            .filter(|e| e.ref_name == ref_name)
            .cloned()
            .collect())
    }
}
