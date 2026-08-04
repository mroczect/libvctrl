use crate::domain::commit::Commit;
use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::domain::tree::Tree;
use crate::error::VctrlError;

pub trait ObjectStore {
    fn put(&mut self, hash: &Hash, obj: &Object) -> Result<(), VctrlError>;
    fn get(&self, hash: &Hash) -> Result<Option<Object>, VctrlError>;
    fn exists(&self, hash: &Hash) -> Result<bool, VctrlError>;
}

pub trait RefStore {
    fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError>;
    fn get_ref(&self, name: &str) -> Result<Option<Hash>, VctrlError>;
    fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError>;
    fn set_head(&mut self, target: &str) -> Result<(), VctrlError>;
    fn head(&self) -> Result<Option<Hash>, VctrlError>;
    fn head_ref_name(&self) -> Result<Option<String>, VctrlError>;
    fn list_refs(&self, prefix: &str) -> Result<Vec<String>, VctrlError>;
}

pub trait ObjectStoreExt {
    fn get_commit(&self, hash: &Hash) -> Result<Commit, VctrlError>;
    fn get_tree(&self, hash: &Hash) -> Result<Tree, VctrlError>;
    fn get_blob(&self, hash: &Hash) -> Result<Vec<u8>, VctrlError>;
}

impl<T: ObjectStore + ?Sized> ObjectStoreExt for T {
    fn get_commit(&self, hash: &Hash) -> Result<Commit, VctrlError> {
        match self.get(hash)? {
            Some(Object::Commit(c)) => Ok(*c),
            _ => Err(VctrlError::NotFound(format!("commit '{}' not found", hash))),
        }
    }
    fn get_tree(&self, hash: &Hash) -> Result<Tree, VctrlError> {
        match self.get(hash)? {
            Some(Object::Tree(t)) => Ok(t),
            _ => Err(VctrlError::NotFound(format!("tree '{}' not found", hash))),
        }
    }
    fn get_blob(&self, hash: &Hash) -> Result<Vec<u8>, VctrlError> {
        match self.get(hash)? {
            Some(Object::Blob(b)) => Ok(b.into_bytes()),
            _ => Err(VctrlError::NotFound(format!("blob '{}' not found", hash))),
        }
    }
}
