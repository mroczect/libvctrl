//! Hasher trait.

use crate::errors::VctrlError;
use crate::types::Hash;
use std::io::Read;

/// Trait for computing hash values.
pub trait Hasher: Send + Sync {
    /// Returns the hash of the data read from the given reader.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if hashing fails.
    fn hash<R: Read + Send>(&self, reader: R) -> Result<Hash, VctrlError>;
}
