use libvctrl_handler::{HASH_LENGTH, VctrlError};






pub const fn validate_hash_bytes(bytes: &[u8]) -> Result<(), VctrlError> {
    if bytes.len() != HASH_LENGTH {
        return Err(VctrlError::InvalidHashLength(bytes.len()));
    }
    Ok(())
}
