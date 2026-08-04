use crate::domain::commit::Commit;
use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::domain::tag::Tag;
use crate::domain::tree::Tree;
use crate::error::VctrlError;
use crate::storage::traits::ObjectStore;

pub trait ObjectStoreExt {
    fn get_commit(&self, hash: &Hash) -> Result<Commit, VctrlError>;
    fn get_tree(&self, hash: &Hash) -> Result<Tree, VctrlError>;
    fn get_blob_bytes(&self, hash: &Hash) -> Result<Vec<u8>, VctrlError>;
    fn get_tag(&self, hash: &Hash) -> Result<Tag, VctrlError>;
    fn require_exists(&self, hash: &Hash) -> Result<(), VctrlError>;
}

impl ObjectStoreExt for dyn ObjectStore + '_ {
    fn get_commit(&self, hash: &Hash) -> Result<Commit, VctrlError> {
        match self.get(hash)? {
            Some(Object::Commit(c)) => Ok(*c),
            Some(obj) => Err(VctrlError::Other(format!(
                "expected commit at '{}' but found {}",
                hash,
                obj.obj_type()
            ))),
            None => Err(VctrlError::NotFound(format!(
                "commit '{}' not found",
                hash
            ))),
        }
    }

    fn get_tree(&self, hash: &Hash) -> Result<Tree, VctrlError> {
        match self.get(hash)? {
            Some(Object::Tree(t)) => Ok(t),
            Some(obj) => Err(VctrlError::Other(format!(
                "expected tree at '{}' but found {}",
                hash,
                obj.obj_type()
            ))),
            None => Err(VctrlError::NotFound(format!(
                "tree '{}' not found",
                hash
            ))),
        }
    }

    fn get_blob_bytes(&self, hash: &Hash) -> Result<Vec<u8>, VctrlError> {
        match self.get(hash)? {
            Some(Object::Blob(b)) => Ok(b.into_bytes()),
            Some(obj) => Err(VctrlError::Other(format!(
                "expected blob at '{}' but found {}",
                hash,
                obj.obj_type()
            ))),
            None => Err(VctrlError::NotFound(format!(
                "blob '{}' not found",
                hash
            ))),
        }
    }

    fn get_tag(&self, hash: &Hash) -> Result<Tag, VctrlError> {
        match self.get(hash)? {
            Some(Object::Tag(t)) => Ok(*t),
            Some(obj) => Err(VctrlError::Other(format!(
                "expected tag at '{}' but found {}",
                hash,
                obj.obj_type()
            ))),
            None => Err(VctrlError::NotFound(format!(
                "tag '{}' not found",
                hash
            ))),
        }
    }

    fn require_exists(&self, hash: &Hash) -> Result<(), VctrlError> {
        if self.exists(hash)? {
            Ok(())
        } else {
            Err(VctrlError::NotFound(format!(
                "object '{}' not found",
                hash
            )))
        }
    }
}
