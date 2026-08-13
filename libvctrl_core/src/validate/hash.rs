//! Hash validation utilities for `libvctrl_core`.
//!
//! # Purpose
//!
//! This module provides utility functions to validate the structural integrity
//! of raw cryptographic hashes before they are converted into strongly-typed
//! [`Hash`](libvctrl_handler::Hash) objects. The single exported function,
//! [`validate_hash_bytes`], ensures that any byte slice intended to represent
//! a hash has exactly the required length.
//!
//! # Design Rationale
//!
//! - **Early failure**: By validating the byte length before attempting to
//!   construct a [`Hash`](libvctrl_handler::Hash), the system fails fast and
//!   provides clear error messages, preventing panics in downstream code.
//! - **Compile-time capability**: The validation function is a `const fn`,
//!   allowing it to be used in `const` evaluation contexts to verify static
//!   hash arrays at compile time. This is useful for checking hardcoded
//!   hashes in configuration or test vectors without runtime overhead.
//! - **Decoupling**: This logic is separated from the `Hash` constructor
//!   itself to keep the data type pure and allow callers to perform
//!   pre-checks if they are interacting with untrusted byte streams.
//! - **Reusability**: Other modules can call this validator before constructing
//!   a [`Hash`](libvctrl_handler::Hash) to provide custom error handling,
//!   logging, or context without repeating the length check.
//!
//! # Relationship to `libvctrl_handler`
//!
//! The [`Hash`](libvctrl_handler::Hash) type in `libvctrl_handler` already
//! performs its own validation inside
//! [`Hash::from_bytes`](libvctrl_handler::Hash::from_bytes). This module does
//! not replace that internal check; rather, it exposes the same invariant as
//! a standalone function. This allows callers to validate data *before*
//! entering a context where constructing a `Hash` would otherwise fail, or
//! where they want a more explicit control flow.
//!
//! # Security Considerations
//!
//! The function only checks the length of the byte slice. It does not verify
//! the cryptographic strength or origin of the bytes. For content-addressing
//! purposes, the caller is responsible for obtaining the hash from a trusted
//! source or computing it via a [`Hasher`](libvctrl_handler::Hasher). This
//! validator acts as the first structural gate, not as a cryptographic
//! guarantee.
//!
//! # Performance
//!
//! The function performs an O(1) comparison between the slice length and the
//! constant [`HASH_LENGTH`](libvctrl_handler::HASH_LENGTH). It does not
//! iterate over the bytes or allocate any memory. In a `const` context, the
//! check is evaluated entirely at compile time, producing zero runtime cost.
//!
//! # When to Use
//!
//! - Before calling [`Hash::from_bytes`](libvctrl_handler::Hash::from_bytes)
//!   when you want a custom error path or need to log the specific invalid
//!   length.
//! - Inside parsers or decoders that read hashes from a binary format and
//!   want to validate fields incrementally.
//! - In `const` contexts where a static hash must be verified to have the
//!   correct length before use.
//!
//! # Examples
//!
//! Validating a correctly sized slice:
//!
//! ```
//! use libvctrl_core::validate::hash::validate_hash_bytes;
//! use libvctrl_handler::HASH_LENGTH;
//!
//! let valid_bytes = [0u8; HASH_LENGTH];
//! assert!(validate_hash_bytes(&valid_bytes).is_ok());
//! ```
//!
//! Validating an incorrectly sized slice:
//!
//! ```
//! use libvctrl_core::validate::hash::validate_hash_bytes;
//! use libvctrl_handler::VctrlError;
//!
//! let invalid_bytes = [0u8; 32];
//! let result = validate_hash_bytes(&invalid_bytes);
//!
//! assert!(matches!(result, Err(VctrlError::InvalidHashLength(32))));
//! ```

use libvctrl_handler::{HASH_LENGTH, VctrlError};

/// Validates that a byte slice is exactly [`HASH_LENGTH`] bytes long.
///
/// # Purpose
///
/// This function acts as a gatekeeper to ensure that any byte slice intended
/// to represent a [`Hash`](libvctrl_handler::Hash) meets the strict length
/// invariant (64 bytes) required by the system. It returns `Ok(())` if the
/// length is correct, or an error describing the mismatch otherwise.
///
/// # Design Rationale
///
/// - **`const fn`**: Being a `const fn` allows this check to be evaluated
///   during compilation if the inputs are known constants. This is useful for
///   verifying hardcoded hashes in configuration or test vectors.
/// - **Pre-conditions check**: It is often used as a pre-check before calling
///   [`Hash::from_bytes`](libvctrl_handler::Hash::from_bytes) to provide
///   custom error handling or logging before the actual conversion.
/// - **Single responsibility**: The function performs exactly one check and
///   does not attempt to construct a [`Hash`](libvctrl_handler::Hash). This
///   keeps the logic simple and composable.
///
/// # Internal Mechanism
///
/// The function compares the length of the provided slice to the constant
/// [`HASH_LENGTH`](libvctrl_handler::HASH_LENGTH). If the lengths differ, it
/// returns [`VctrlError::InvalidHashLength`] containing the actual length.
/// The comparison and branch are trivial and compile to a handful of CPU
/// instructions.
///
/// # Errors
///
/// Returns
/// [`VctrlError::InvalidHashLength`](libvctrl_handler::VctrlError::InvalidHashLength)
/// if the length of `bytes` is not exactly equal to
/// [`HASH_LENGTH`](libvctrl_handler::HASH_LENGTH) (64). The error payload is
/// the actual length of the input slice, allowing callers to report the
/// mismatch precisely.
///
/// # Panics
///
/// This function never panics. It handles all possible inputs gracefully by
/// returning a [`Result`].
///
/// # Examples
///
/// Validating a correctly sized slice:
///
/// ```
/// use libvctrl_core::validate::hash::validate_hash_bytes;
/// use libvctrl_handler::HASH_LENGTH;
///
/// let valid_bytes = [0u8; HASH_LENGTH];
/// assert!(validate_hash_bytes(&valid_bytes).is_ok());
/// ```
///
/// Validating an incorrectly sized slice:
///
/// ```
/// use libvctrl_core::validate::hash::validate_hash_bytes;
/// use libvctrl_handler::VctrlError;
///
/// let invalid_bytes = [0u8; 32];
/// let result = validate_hash_bytes(&invalid_bytes);
///
/// assert!(matches!(result, Err(VctrlError::InvalidHashLength(32))));
/// ```
pub const fn validate_hash_bytes(bytes: &[u8]) -> Result<(), VctrlError> {
    if bytes.len() != HASH_LENGTH {
        return Err(VctrlError::InvalidHashLength(bytes.len()));
    }
    Ok(())
}
