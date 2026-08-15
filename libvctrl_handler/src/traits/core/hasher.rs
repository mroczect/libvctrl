use crate::errors::VctrlError;
use crate::types::hash::Hash;

pub trait Hasher {
    fn hash(&self, data: &[u8]) -> Result<Hash, VctrlError>;
}
