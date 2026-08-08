//! Hash validation utilities.

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
