use crate::errors::VctrlError;
use crate::types::hash::Hash;

pub trait Transport {
    fn fetch_object(&self, hash: &Hash) -> Result<Vec<u8>, VctrlError>;

    fn push_object(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError>;
}
