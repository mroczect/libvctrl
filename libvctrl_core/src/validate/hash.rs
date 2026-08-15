use libvctrl_handler::{HASH_LENGTH, VctrlError};

/// Validates that a byte slice is exactly `HASH_LENGTH` bytes long.
///
/// # Errors
///
/// Returns [`VctrlError::InvalidHashLength`] if the slice length does not match `HASH_LENGTH`.
pub const fn validate_hash_bytes(bytes: &[u8]) -> Result<(), VctrlError> {
    if bytes.len() != HASH_LENGTH {
        return Err(VctrlError::InvalidHashLength(bytes.len()));
    }
    Ok(())
}
