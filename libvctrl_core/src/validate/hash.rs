//! Hash validation utilities for `libvctrl_core`.
//!
//! # Purpose
//! This module provides utility functions to validate the structural integrity
//! of raw cryptographic hashes before they are converted into strongly-typed
//! [`Hash`](libvctrl_handler::Hash) objects.
//!
//! # Design rationale
//! - **Early Failure**: By validating the byte length before attempting to
//!   construct a [`Hash`](libvctrl_handler::Hash), the system fails fast and
//!   provides clear error messages, preventing panics in downstream code.
//! - **Compile-time Capability**: The validation function is a `const fn`,
//!   allowing it to be used in `const` evaluation contexts to verify static
//!   hash arrays at compile time.
//! - **Decoupling**: This logic is separated from the `Hash` constructor itself
//!   to keep the data type pure and allow callers to perform pre-checks if
//!   they are interacting with untrusted byte streams.

use libvctrl_handler::{HASH_LENGTH, VctrlError};

/// Validates that a byte slice is exactly [`HASH_LENGTH`] bytes long.
///
/// # Purpose
/// This function acts as a gatekeeper to ensure that any byte slice intended
/// to represent a [`Hash`](libvctrl_handler::Hash) meets the strict length
/// invariant (64 bytes) required by the system.
///
/// # Design rationale
/// - **`const fn`**: Being a `const fn` allows this check to be evaluated
///   during compilation if the inputs are known constants. This is useful for
///   verifying hardcoded hashes in configuration or test vectors.
/// - **Pre-conditions Check**: It is often used as a pre-check before calling
///   [`Hash::from_bytes`](libvctrl_handler::Hash::from_bytes) to provide custom
///   error handling or logging before the actual conversion.
///
/// # Internal mechanism
/// The function performs an `O(1)` comparison between the length of the
/// provided slice and the constant [`HASH_LENGTH`]. If they differ, it returns
/// a [`VctrlError::InvalidHashLength`] containing the incorrect length.
///
/// # Errors
/// Returns [`VctrlError::InvalidHashLength`](libvctrl_handler::VctrlError::InvalidHashLength)
/// if the length of `bytes` is not exactly equal to [`HASH_LENGTH`] (64).
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
/// use libvctrl_handler::{HASH_LENGTH, VctrlError};
///
/// let invalid_bytes = [0u8; 32]; // Wrong length
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
