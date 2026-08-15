//! Signing trait.

use crate::errors::VctrlError;

/// Trait for signing data.
pub trait Signer {
    /// Signs the given data and returns the signature.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if signing fails.
    fn sign(&mut self, data: &[u8]) -> Result<Vec<u8>, VctrlError>;
}
