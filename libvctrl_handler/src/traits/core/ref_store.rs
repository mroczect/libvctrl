use crate::errors::VctrlError;
use crate::types::Hash;

pub trait RefStore: Send + Sync {
    type RefsIterator: Iterator<Item = Result<String, VctrlError>> + Send;

    fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError>;

    fn get_ref(&self, name: &str) -> Result<Hash, VctrlError>;

    fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError>;

    fn list_refs(&self) -> Result<Self::RefsIterator, VctrlError>;
}
