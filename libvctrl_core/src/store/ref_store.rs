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
        crate::validate::name::validate_ref_name(name)?;
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
