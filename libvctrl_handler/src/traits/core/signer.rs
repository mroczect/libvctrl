//! Cryptographic signing of data.
//!
//! # Purpose
//!
//! This module defines the [`Signer`] trait, which abstracts the process of
//! producing cryptographic signatures over arbitrary byte slices. In a
//! version control system, signatures are typically used to attest to the
//! authenticity and integrity of [`Commit`](crate::Commit) and
//! [`Tag`](crate::Tag) objects, allowing users to verify that an object was
//! indeed created by the claimed author.
//!
//! # Design Rationale
//!
//! Signing is separated into a trait for several reasons:
//!
//! - **Algorithm agnosticism**: Different deployments may choose different
//!   signature schemes (Ed25519, RSA, ECDSA, etc.). The trait contract only
//!   requires that a signer transforms bytes into a signature, so the rest
//!   of the system remains independent of the specific algorithm.
//! - **Testability**: Dummy or deterministic signers can be injected in unit
//!   tests, avoiding the need for actual private keys or hardware signing
//!   modules.
//! - **Stateful signing**: The trait method takes `&mut self` because signing
//!   often requires mutable internal state. For example, hardware security
//!   modules may maintain session state, or deterministic nonce generation
//!   may need a counter to prevent reuse.
//! - **Flexible output**: The signature is returned as a `Vec<u8>` because
//!   different algorithms produce signatures of varying lengths. Returning a
//!   byte vector keeps the trait fully generic.
//!
//! # Why `&mut self`?
//!
//! The [`Signer::sign`] method takes `&mut self` rather than `&self`. This
//! design choice reflects the fact that signing often involves mutable
//! internal state:
//!
//! - Cryptographic libraries may use a stateful random number generator or
//!   a nonce counter that must be advanced after each signature.
//! - Hardware security modules may maintain session keys that change over
//!   time.
//! - Some algorithms require precomputation caches that are built lazily.
//!
//! Using `&mut self` ensures that implementations can safely manage such
//! state without interior mutability or synchronization overhead.
//!
//! # Error Handling
//!
//! The signing operation may fail for various reasons, including:
//!
//! - A missing or invalid private key.
//! - Hardware module unavailability.
//! - Algorithm-specific constraints (e.g., message too long for RSA).
//!
//! The trait therefore returns [`Result<Vec<u8>, VctrlError>`], allowing
//! implementations to report failures through the crate's unified error type.
//! The most common error variant is
//! [`VctrlError::Other`](crate::VctrlError::Other), which can carry a
//! descriptive message.
//!
//! # Internal Mechanism
//!
//! A typical implementation will:
//!
//! 1. Access the private key or signing context stored in `self`.
//! 2. Feed the provided `data` into the signature algorithm.
//! 3. Return the resulting signature as a byte vector.
//!
//! The exact steps depend on the algorithm, but the trait ensures that the
//! interface remains consistent across all backends.
//!
//! # Examples
//!
//! A simple deterministic signer that returns the data itself as the
//! signature:
//!
//! ```
//! use libvctrl_handler::{Signer, VctrlError};
//!
//! struct IdentitySigner;
//!
//! impl Signer for IdentitySigner {
//!     fn sign(&mut self, data: &[u8]) -> Result<Vec<u8>, VctrlError> {
//!         Ok(data.to_vec())
//!     }
//! }
//!
//! let mut signer = IdentitySigner;
//! let signature = signer.sign(b"hello").unwrap();
//! assert_eq!(signature, b"hello");
//! ```

use crate::errors::VctrlError;

/// Defines the interface for signing data cryptographically.
///
/// # Purpose
///
/// A `Signer` produces a cryptographic signature over a byte slice, typically
/// to attest to the authenticity of a [`Commit`](crate::Commit) or
/// [`Tag`](crate::Tag). The trait is intentionally minimal: it only specifies
/// the signing operation itself, leaving algorithm selection and key
/// management to the implementation.
///
/// # Design Rationale
///
/// The trait returns a `Vec<u8>` to remain agnostic to the underlying
/// signature algorithm (e.g., Ed25519, RSA, ECDSA). The input is a `&[u8]`
/// slice, allowing the caller to sign any serialized object or raw bytes
/// without requiring a specific type. The method takes `&mut self` because
/// signing often involves stateful operations (nonce generation, session
/// keys, or hardware module sessions).
///
/// # Why Not a Generic Input?
///
/// The input is a plain byte slice rather than a generic type to keep the
/// trait simple and avoid imposing serialization requirements on the data
/// being signed. Callers are responsible for converting their objects to
/// bytes (e.g., using an [`Encoder`](crate::Encoder)) before calling
/// [`sign`](Signer::sign).
///
/// # Why `Result<Vec<u8>>`?
///
/// Signing is fallible because the private key may be missing, the hardware
/// module may be unavailable, or the algorithm may reject the input length.
/// The return type captures both success (as a byte vector) and failure (as
/// a [`VctrlError`]).
///
/// # How It Works Internally
///
/// An implementation receives the raw bytes and returns a signature. It is
/// responsible for:
///
/// - Accessing the private key or signing context.
/// - Applying the signature algorithm.
/// - Returning the signature bytes in the algorithm's native encoding.
///
/// # Examples
///
/// A dummy signer that signs by simply copying the input:
///
/// ```
/// use libvctrl_handler::{Signer, VctrlError};
///
/// struct DummySigner;
///
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
    /// # Purpose
    ///
    /// This method is the core contract of the [`Signer`] trait. It takes an
    /// arbitrary byte slice and produces a cryptographic signature that can
    /// later be verified using a corresponding
    /// [`Verifier`](crate::Verifier) implementation.
    ///
    /// # Arguments
    ///
    /// * `data` - The raw bytes to sign. Typically this is the serialized
    ///   representation of a [`Commit`](crate::Commit) or
    ///   [`Tag`](crate::Tag), but any byte slice is accepted.
    ///
    /// # Returns
    ///
    /// Returns `Ok(signature)` where `signature` is the signature as a
    /// byte vector. The length and format of the signature are determined
    /// by the specific signing algorithm.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::Other`] if the signing process fails, for
    /// example because the private key is missing, the hardware module is
    /// unavailable, or the algorithm rejects the input length. The error
    /// message should provide additional context.
    ///
    /// # How It Works Internally
    ///
    /// The implementation receives the input slice, performs the signature
    /// operation using its internal key material, and returns the resulting
    /// signature. The method takes `&mut self` to allow stateful operations
    /// such as nonce counters or session management.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use libvctrl_handler::{Signer, VctrlError};
    /// # struct SignerImpl;
    /// # impl Signer for SignerImpl {
    /// #     fn sign(&mut self, d: &[u8]) -> Result<Vec<u8>, VctrlError> {
    /// #         Ok(d.to_vec())
    /// #     }
    /// # }
    /// let mut signer = SignerImpl;
    /// let sig = signer.sign(b"data").unwrap();
    /// assert!(!sig.is_empty());
    /// ```
    ///
    /// Signing a serialized commit-like payload:
    ///
    /// ```
    /// # use libvctrl_handler::{Signer, VctrlError};
    /// # struct SignerImpl;
    /// # impl Signer for SignerImpl {
    /// #     fn sign(&mut self, d: &[u8]) -> Result<Vec<u8>, VctrlError> {
    /// #         Ok(d.to_vec())
    /// #     }
    /// # }
    /// let payload = b"commit hash: abcdef";
    /// let mut signer = SignerImpl;
    /// let signature = signer.sign(payload).unwrap();
    /// assert_eq!(signature, payload);
    /// ```
    fn sign(&mut self, data: &[u8]) -> Result<Vec<u8>, VctrlError>;
}
