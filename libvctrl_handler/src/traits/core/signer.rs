//! Signing trait.
//!
//! # Architecture
//! This module defines the abstract contract for cryptographically signing data
//! (e.g., commits or tags). By abstracting the signing mechanism into a trait,
//! the crate decouples its security logic from the specific cryptographic backend.
//! This allows consumers to plug in different implementations, such as GPG, SSH,
//! or cloud-based Key Management Services (KMS), without altering the core VCS engine.
//!
//! # Design Rationale: Stateful Signing
//! The `sign` method requires `&mut self`. This is a deliberate design choice
//! because cryptographic signing is often stateful. A backend might need to consume
//! a one-time-use nonce, update an internal counter for replay protection, or acquire
//! an exclusive lock on a hardware security module (HSM). Forcing `&mut self` at the
//! trait level ensures that backends have the flexibility to implement these requirements
//! safely without resorting to interior mutability (`Mutex` or `RefCell`).

use crate::errors::VctrlError;

/// Trait for signing data.
///
/// # Why this exists
/// Provides a unified interface for generating cryptographic signatures. In Git,
/// signed commits and tags verify the identity of the author. This trait allows
/// the engine to delegate the complex cryptography to a dedicated backend, ensuring
/// that the core logic remains focused on object manipulation and graph traversal.
///
/// # How it works
/// The implementor receives a `key_id` (which could be a GPG key fingerprint, an
/// SSH key path, or a KMS URI) and the raw `data` to be signed. The backend locates
/// the private key, performs the cryptographic signing operation, and returns the
/// resulting signature as an owned `Vec<u8>`.
///
/// # Design Rationale: Owned `Vec<u8>` Return
/// The signature is returned as an owned `Vec<u8>` rather than a fixed-size array.
/// Different signing algorithms produce different signature lengths (e.g., RSA signatures
/// are significantly larger than EdDSA signatures). Returning a vector accommodates
/// all algorithms uniformly.
///
/// # Examples
///
/// Implementing the trait for a mock signer:
///
/// ```
/// # use libvctrl_handler::traits::core::signer::Signer;
/// # use libvctrl_handler::VctrlError;
/// #
/// struct MockSigner;
///
/// impl Signer for MockSigner {
///     fn sign(&mut self, key_id: &str, data: &[u8]) -> Result<Vec<u8>, VctrlError> {
///         // A real implementation would use a private key here.
///         let mut signature = Vec::new();
///         signature.extend_from_slice(key_id.as_bytes());
///         signature.push(b':');
///         signature.extend_from_slice(data);
///         Ok(signature)
///     }
/// }
///
/// let mut signer = MockSigner;
/// let sig = signer.sign("ABCDEFG12345", b"commit data")?;
/// assert_eq!(sig, b"ABCDEFG12345:commit data");
/// # Ok::<(), VctrlError>(())
/// ```
pub trait Signer: Send + Sync {
    /// Signs the given data with the specified key ID and returns the signature.
    ///
    /// # How it works
    /// Resolves the `key_id` to a private key within the backend's keyring. It then
    /// applies the signing algorithm (e.g., RSA-SHA256, Ed25519) to the provided
    /// `data` slice. The resulting cryptographic signature is returned as an owned
    /// byte vector.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if:
    /// - The `key_id` cannot be found in the keyring.
    /// - The private key requires a passphrase that could not be provided.
    /// - The underlying cryptographic operation fails.
    /// - An I/O error occurs (e.g., communicating with a hardware token).
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::signer::Signer;
    /// # use libvctrl_handler::VctrlError;
    /// # struct MockSigner;
    /// # impl Signer for MockSigner {
    /// #     fn sign(&mut self, key_id: &str, data: &[u8]) -> Result<Vec<u8>, VctrlError> {
    /// #         Ok(data.to_vec())
    /// #     }
    /// # }
    /// let mut signer = MockSigner;
    /// let data = b"data to sign";
    /// let signature = signer.sign("key-id", data)?;
    /// assert_eq!(signature, data);
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn sign(&mut self, key_id: &str, data: &[u8]) -> Result<Vec<u8>, VctrlError>;
}
