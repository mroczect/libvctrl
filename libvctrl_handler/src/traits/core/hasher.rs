use std::io::Read;

use crate::errors::VctrlError;
use crate::types::Hash;

pub trait Hasher: Send + Sync {
    fn hash<R: Read + Send>(&self, reader: R) -> Result<Hash, VctrlError>;
}
