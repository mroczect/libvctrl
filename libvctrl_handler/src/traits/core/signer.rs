//! Cryptographic signing of data.

use crate::errors::VctrlError;

/// Defines the interface for signing data cryptographically.
///
/// # Purpose
///
/// A `Signer` produces a cryptographic signature over a byte slice, typically
/// to attest to the authenticity of a [`Commit`] or [`Tag`].
///
/// # Design Rationale
///
/// The trait returns a `Vec<u8>` to remain agnostic to the underlying
/// signature algorithm (e.g., Ed25519, RSA).
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Signer, VctrlError};
///
/// struct DummySigner;
/// impl Signer for DummySigner {
///     fn sign(&mut self, data: &[u8]) -> Result<Vec<u8>, VctrlError> {
///         Ok(data.to_vec())
///     }
/// }
///
/// let mut signer = DummySigner;
/// let sig = signer.sign(b"msg").unwrap();
/// assert_eq!(sig, b"msg");
/// ```
pub trait Signer {
    /// Signs the provided data, returning the signature as a byte vector.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::Other`] if the signing process fails (e.g.,
    /// missing private key).
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Signer, VctrlError};
    /// # struct SignerImpl;
    /// # impl Signer for SignerImpl {
    /// #     fn sign(&mut self, d: &[u8]) -> Result<Vec<u8>, VctrlError> { Ok(d.to_vec()) }
    /// # }
    /// let mut signer = SignerImpl;
    /// let sig = signer.sign(b"data").unwrap();
    /// assert!(!sig.is_empty());
    /// ```
    fn sign(&mut self, data: &[u8]) -> Result<Vec<u8>, VctrlError>;
}
