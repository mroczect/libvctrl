use crate::errors::VctrlError;
use crate::types::hash::Hash;

pub trait RefStore {
    type RefsIterator: Iterator<Item = Result<String, VctrlError>>;

    fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError>;

    fn get_ref(&self, name: &str) -> Result<Hash, VctrlError>;

    fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError>;

    fn list_refs(&self) -> Result<Self::RefsIterator, VctrlError>;
}
