//! Verification trait.

use crate::errors::VctrlError;

/// Trait for verifying signatures.
pub trait Verifier {
    /// Verifies data against a signature.
    ///
    /// Returns `Ok(true)` if the signature is valid for the data.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if verification cannot be performed.
    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, VctrlError>;
}
