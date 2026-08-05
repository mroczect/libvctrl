use crate::domain::commit::Commit;
use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::domain::tree::Tree;
use crate::error::VctrlError;
use crate::hashing::HashVerifier;

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

    fn get_verified(
        &self,
        hash: &Hash,
        encoder: &dyn crate::codec::Encoder,
        hasher: &dyn crate::hashing::Hasher,
    ) -> Result<Object, VctrlError>;
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

    fn get_verified(
        &self,
        hash: &Hash,
        encoder: &dyn crate::codec::Encoder,
        hasher: &dyn crate::hashing::Hasher,
    ) -> Result<Object, VctrlError> {
        let obj = self
            .get(hash)?
            .ok_or_else(|| VctrlError::NotFound(format!("object '{}' not found", hash)))?;
        match &obj {
            Object::Blob(b) => {
                if !hasher.verify_blob(hash, b.as_bytes()) {
                    return Err(VctrlError::Corrupted(format!(
                        "blob hash mismatch: {}",
                        hash
                    )));
                }
            }
            Object::Tree(t) => {
                let mut buf = Vec::new();
                encoder.encode_tree(t, &mut buf)?;
                if !hasher.verify_tree_encoded(hash, &buf) {
                    return Err(VctrlError::Corrupted(format!(
                        "tree hash mismatch: {}",
                        hash
                    )));
                }
            }
            Object::Commit(c) => {
                let mut buf = Vec::new();
                encoder.encode_commit(c, &mut buf)?;
                if !hasher.verify_commit_encoded(hash, &buf) {
                    return Err(VctrlError::Corrupted(format!(
                        "commit hash mismatch: {}",
                        hash
                    )));
                }
            }
            Object::Tag(t) => {
                let mut buf = Vec::new();
                encoder.encode_tag(t, &mut buf)?;
                if !hasher.verify_tag_encoded(hash, &buf) {
                    return Err(VctrlError::Corrupted(format!(
                        "tag hash mismatch: {}",
                        hash
                    )));
                }
            }
        }
        Ok(obj)
    }
}
