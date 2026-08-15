//! Verification trait.

use crate::errors::VctrlError;

/// Trait for verifying signatures.
pub trait Verifier: Send + Sync {
    /// Verifies data against a signature using the specified key ID.
    ///
    /// Returns `Ok(true)` if the signature is valid for the data.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if verification cannot be performed.
    fn verify(&self, key_id: &str, data: &[u8], signature: &[u8]) -> Result<bool, VctrlError>;
}
