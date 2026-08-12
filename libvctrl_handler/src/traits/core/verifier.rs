//! Verification of cryptographic signatures.

use crate::errors::VctrlError;

/// Defines the interface for verifying cryptographic signatures.
///
/// # Purpose
///
/// A `Verifier` checks whether a given byte slice and signature pair are valid
/// according to a specific cryptographic key.
///
/// # Design Rationale
///
/// Returns `Result<bool, VctrlError>` rather than just `bool` to allow for
/// verification failures that are not strictly boolean (e.g., malformed
/// signature inputs or internal cryptographic errors).
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Verifier, VctrlError};
///
/// struct DummyVerifier;
/// impl Verifier for DummyVerifier {
///     fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, VctrlError> {
///         Ok(data == signature)
///     }
/// }
///
/// let verifier = DummyVerifier;
/// assert!(verifier.verify(b"msg", b"msg").unwrap());
/// assert!(!verifier.verify(b"msg", b"bad").unwrap());
/// ```
pub trait Verifier {
    /// Verifies a signature against the provided data.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::Other`] if the verification process encounters
    /// an internal error (e.g., malformed signature).
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Verifier, VctrlError};
    /// # struct VerifierImpl;
    /// # impl Verifier for VerifierImpl {
    /// #     fn verify(&self, d: &[u8], s: &[u8]) -> Result<bool, VctrlError> { Ok(d == s) }
    /// # }
    /// let verifier = VerifierImpl;
    /// assert!(verifier.verify(b"data", b"data").unwrap());
    /// assert!(!verifier.verify(b"data", b"tampered").unwrap());
    /// ```
    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, VctrlError>;
}
