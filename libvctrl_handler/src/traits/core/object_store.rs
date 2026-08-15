use crate::errors::VctrlError;
use crate::types::hash::Hash;
use std::io::Read;

pub trait ObjectStore {
    fn put(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError>;

    fn get(&self, hash: &Hash) -> Result<Box<dyn Read + '_>, VctrlError>;

    fn delete(&mut self, hash: &Hash) -> Result<(), VctrlError>;

    fn exists(&self, hash: &Hash) -> Result<bool, VctrlError>;
}
