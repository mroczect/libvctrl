//! Hash validation utilities.
//!
//! # Architecture
//! This module provides standalone validation for byte slices intended to be used
//! as Git object hashes. It ensures that data read from untrusted sources (like
//! network packfiles) is the correct length before attempting to construct a
//! [`Hash`](crate::Hash) type.
//!
//! # Design Rationale: Compile-Time Evaluation
//! The primary validation function is implemented as a `const fn`. This is a
//! critical architectural decision: it allows validation to occur at compile time
//! if the input byte slice is a known constant. This shifts the computational
//! overhead to the compiler, achieving true zero-cost runtime validation for
//! static data.

use crate::constants::HASH_LENGTH;
use crate::errors::VctrlError;

/// Validates that a byte slice is exactly `HASH_LENGTH` bytes long.
///
/// # Why this exists
/// Git's SHA-512 implementation requires exactly 64 bytes. Passing a slice of
/// incorrect length to a hash constructor would either cause a runtime panic
/// (if using fixed-size array conversion) or silently produce an invalid hash.
/// This function provides a safe, fallible boundary to verify length before
/// memory allocation or cryptographic processing.
///
/// # How it works
/// As a `const fn`, this can be evaluated by the compiler. If the input is a
/// static byte array (e.g., `b"..."`), the compiler can resolve the `Result`
/// at compile time, eliminating the runtime branch entirely.
///
/// # Errors
///
/// Returns [`VctrlError::InvalidHashLength`] if the slice length does not match
/// [`HASH_LENGTH`].
///
/// # Examples
///
/// Validating a correctly sized slice:
///
/// ```
/// # use libvctrl_handler::validation::validate_hash_bytes;
/// let valid_hash = [0_u8; 64];
/// assert!(validate_hash_bytes(&valid_hash).is_ok());
/// ```
///
/// Handling an invalid slice:
///
/// ```
/// # use libvctrl_handler::validation::validate_hash_bytes;
/// # use libvctrl_handler::VctrlError;
/// let invalid_hash = [0_u8; 32];
/// let result = validate_hash_bytes(&invalid_hash);
/// assert!(matches!(result, Err(VctrlError::InvalidHashLength(32))));
/// ```
pub const fn validate_hash_bytes(bytes: &[u8]) -> Result<(), VctrlError> {
    if bytes.len() != HASH_LENGTH {
        return Err(VctrlError::InvalidHashLength(bytes.len()));
    }
    Ok(())
}
