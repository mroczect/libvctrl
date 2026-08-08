//! Name validation utilities.

use libvctrl_handler::{MAX_NAME_LENGTH, VctrlError};

/// Validates a name according to the fundamental contracts.
///
/// A valid name:
/// - Is not empty.
/// - Does not exceed `MAX_NAME_LENGTH` bytes.
///
/// This function is the single point of truth for name validation in
/// `libvctrl_core`. All higher-level modules (builders, stores) call it
/// before constructing objects that carry a name.
///
/// # Errors
/// Returns [`VctrlError::InvalidName`] with a descriptive message.
///
/// # Examples
/// ```
/// use libvctrl_core::validate::name::validate_name;
/// assert!(validate_name("hello").is_ok());
/// assert!(validate_name("").is_err());
/// assert!(validate_name(&"a".repeat(300)).is_err());
/// ```
pub fn validate_name(name: &str) -> Result<(), VctrlError> {
    if name.is_empty() {
        return Err(VctrlError::InvalidName("name is empty".into()));
    }
    if name.len() > MAX_NAME_LENGTH {
        // Cannot use format! in const fn, so we provide a static message
        return Err(VctrlError::InvalidName(
            "name exceeds maximum length".into(),
        ));
    }
    Ok(())
}
