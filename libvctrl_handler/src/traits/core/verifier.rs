//! Verification of cryptographic signatures.
//!
//! # Purpose
//!
//! This module defines the [`Verifier`] trait, which abstracts the process
//! of checking whether a given byte slice and a cryptographic signature are
//! valid according to a specific public key or verification context. In a
//! version control system, signature verification is used to confirm the
//! authenticity and integrity of signed objects such as commits and tags.
//!
//! # Design Rationale
//!
//! Verification is separated from signing for several reasons:
//!
//! - **Separation of concerns**: Signing requires private key material,
//!   while verification only requires public information. Separating them
//!   allows distributing verifiers widely without exposing secrets.
//! - **Different lifecycles**: A signer may be short-lived and stateful,
//!   whereas a verifier can often be stateless and shared across threads.
//! - **Testability**: Dummy or deterministic verifiers simplify unit tests
//!   of higher-level integrity checks.
//! - **Flexibility**: Different signature algorithms (Ed25519, RSA, ECDSA)
//!   can be supported by implementing the same trait, keeping the rest of
//!   the system agnostic.
//!
//! # Why `Result<bool>`?
//!
//! The [`Verifier::verify`] method returns `Result<bool, VctrlError>` rather
//! than a plain `bool` to allow distinguishing between:
//!
//! - A valid signature (`Ok(true)`).
//! - An invalid signature (`Ok(false)`).
//! - A failure to perform verification at all (`Err(...)`), for example
//!   because the signature is malformed, the public key is invalid, or an
//!   internal cryptographic error occurred.
//!
//! This design prevents callers from silently treating verification failures
//! as `false` and potentially accepting tampered data without realizing the
//! verifier itself failed.
//!
//! # Internal Mechanism
//!
//! A typical implementation receives the raw data, the signature bytes, and
//! uses its internal public key or verification context to perform the
//! cryptographic check. The exact steps depend on the algorithm, but the
//! trait ensures a uniform interface.
//!
//! # Relationship to [`Signer`]
//!
//! A [`Signer`] produces signatures, and a [`Verifier`]
//! checks them. They are designed as separate traits to reflect real-world
//! security practices where signing and verification use different keys and
//! often different software components.
//!
//! # Examples
//!
//! A simple verifier that compares data and signature for equality:
//!
//! ```
//! use libvctrl_handler::{Verifier, VctrlError};
//!
//! struct EqualityVerifier;
//!
//! impl Verifier for EqualityVerifier {
//!     fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, VctrlError> {
//!         Ok(data == signature)
//!     }
//! }
//!
//! let verifier = EqualityVerifier;
//! assert!(verifier.verify(b"msg", b"msg").unwrap());
//! assert!(!verifier.verify(b"msg", b"bad").unwrap());
//! ```

use crate::errors::VctrlError;

/// Defines the interface for verifying cryptographic signatures.
///
/// # Purpose
///
/// A `Verifier` checks whether a given byte slice and signature pair are
/// valid according to a specific cryptographic key or verification context.
/// It is the counterpart to [`Signer`], which produces
/// signatures. Verification is used to confirm that data has not been
/// altered and was indeed signed by the claimed entity.
///
/// # Design Rationale
///
/// Returns `Result<bool, VctrlError>` rather than just `bool` to allow for
/// verification failures that are not strictly boolean (e.g., malformed
/// signature inputs, invalid public keys, or internal cryptographic errors).
/// This distinction is critical for security-sensitive code: callers can
/// detect whether verification could not be performed and treat such cases
/// differently from a signature that is simply invalid.
///
/// # Why `&self`?
///
/// The method takes `&self` because verification is generally a read-only
/// operation. A verifier holds a public key or verification context that can
/// be safely shared and reused. This also allows a single verifier instance
/// to be used concurrently across threads if the implementation is
/// [`Sync`].
///
/// # Why `&[u8]` for Data and Signature?
///
/// Both parameters are byte slices to keep the interface generic. The data
/// is typically a serialized object or message, and the signature is the
/// algorithm-specific byte encoding. Using slices avoids lifetime
/// constraints and allows verification of any byte sequence.
///
/// # Internal Mechanism
///
/// The implementation receives the raw data and signature bytes, retrieves
/// its public key or verification state, and performs the cryptographic
/// check. It returns `Ok(true)` if the signature is valid, `Ok(false)` if it
/// is not, or an error if the verification process itself fails (for
/// example, because the signature is malformed).
///
/// # Examples
///
/// A dummy verifier that compares data and signature:
///
/// ```
/// use libvctrl_handler::{Verifier, VctrlError};
///
/// struct DummyVerifier;
///
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
    /// # Purpose
    ///
    /// This method is the core contract of the [`Verifier`] trait. It checks
    /// whether the given `signature` is valid for the given `data` according
    /// to the verifier's cryptographic key. The result is a boolean
    /// indicating validity, wrapped in a [`Result`] to allow reporting
    /// verification failures separately from an invalid signature.
    ///
    /// # Arguments
    ///
    /// * `data` - The raw bytes that were signed. This is typically the
    ///   serialized representation of a version control object.
    /// * `signature` - The signature bytes to verify, in the format produced
    ///   by the corresponding [`Signer`].
    ///
    /// # Returns
    ///
    /// Returns:
    ///
    /// - `Ok(true)` if the signature is valid for the data.
    /// - `Ok(false)` if the signature is not valid.
    /// - `Err(...)` if the verification process itself failed (e.g.,
    ///   malformed signature or invalid key).
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::Other`] if the verification process encounters
    /// an internal error, such as a malformed signature, an invalid public
    /// key, or an algorithm-specific failure. The error message should
    /// provide diagnostic context.
    ///
    /// # How It Works Internally
    ///
    /// The implementation receives the data and signature slices, retrieves
    /// its public key or verification state, and performs the cryptographic
    /// verification. On success it returns `Ok(true)`; on a signature
    /// mismatch it returns `Ok(false)`; on any error that prevents
    /// verification it returns an appropriate [`VctrlError`].
    ///
    /// # Examples
    ///
    /// Basic verification:
    ///
    /// ```
    /// # use libvctrl_handler::{Verifier, VctrlError};
    /// # struct VerifierImpl;
    /// # impl Verifier for VerifierImpl {
    /// #     fn verify(&self, d: &[u8], s: &[u8]) -> Result<bool, VctrlError> {
    /// #         Ok(d == s)
    /// #     }
    /// # }
    /// let verifier = VerifierImpl;
    /// assert!(verifier.verify(b"data", b"data").unwrap());
    /// assert!(!verifier.verify(b"data", b"tampered").unwrap());
    /// ```
    ///
    /// Distinguishing invalid signature from verification error:
    ///
    /// ```
    /// # use libvctrl_handler::{Verifier, VctrlError};
    /// # struct StrictVerifier;
    /// # impl Verifier for StrictVerifier {
    /// #     fn verify(&self, d: &[u8], s: &[u8]) -> Result<bool, VctrlError> {
    /// #         if s.is_empty() {
    /// #             return Err(VctrlError::Other("empty signature".to_string()));
    /// #         }
    /// #         Ok(d == s)
    /// #     }
    /// # }
    /// let verifier = StrictVerifier;
    /// assert!(verifier.verify(b"data", b"data").unwrap());
    /// assert!(!verifier.verify(b"data", b"wrong").unwrap());
    /// assert!(verifier.verify(b"data", b"").is_err());
    /// ```
    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, VctrlError>;
}
