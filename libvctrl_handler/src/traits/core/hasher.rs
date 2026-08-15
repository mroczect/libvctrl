//! Hasher trait.

use crate::errors::VctrlError;
use crate::types::Hash;

/// Trait for computing hash values.
pub trait Hasher {
    /// Returns the hash of the given data.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if hashing fails.
    fn hash(&self, data: &[u8]) -> Result<Hash, VctrlError>;
}
