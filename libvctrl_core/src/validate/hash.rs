//! Hash validation utilities.
//!
//! This module provides [`validate_hash_bytes`], a function that checks
//! whether a byte slice has the exact length required for a [`Hash`].
//!
//! # Why validate?
//!
//! The [`Hash`] type enforces its length at construction, so you might
//! wonder why this function exists. It is useful in two scenarios:
//!
//! 1. **Pre‑validation** – you want to check a large number of byte
//!    slices before constructing hashes, to fail early and avoid
//!    partial failures.
//! 2. **Generic code** – when you are writing code that operates on
//!    raw `&[u8]` and want to ensure length compliance without
//!    immediately creating a `Hash`.
//!
//! Most code should use [`Hash::from_bytes`] directly, as it performs
//! the same check and returns a `Hash` on success.
//!
//! # Example
//!
//! ```rust
//! use libvctrl_core::validate::hash::validate_hash_bytes;
//!
//! let valid = [0xAB; 64];
//! assert!(validate_hash_bytes(&valid).is_ok());
//!
//! let invalid = [0xAB; 10];
//! assert!(validate_hash_bytes(&invalid).is_err());
//! ```

use libvctrl_handler::{HASH_LENGTH, VctrlError};

/// Validates that a byte slice has the exact length required for a hash.
///
/// Most code should use [`Hash::from_bytes`] directly, but this function
/// is useful for pre-validating data before constructing a hash in a
/// context where you need to defer the actual construction.
///
/// # Errors
/// Returns [`VctrlError::InvalidHashLength`] if the slice length is incorrect.
///
/// # Examples
/// ```
/// use libvctrl_core::validate::hash::validate_hash_bytes;
/// assert!(validate_hash_bytes(&[0u8; 64]).is_ok());
/// assert!(validate_hash_bytes(&[0u8; 10]).is_err());
/// ```
pub const fn validate_hash_bytes(bytes: &[u8]) -> Result<(), VctrlError> {
    if bytes.len() != HASH_LENGTH {
        return Err(VctrlError::InvalidHashLength(bytes.len()));
    }
    Ok(())
}
