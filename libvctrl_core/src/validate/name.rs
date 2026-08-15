use libvctrl_handler::{MAX_NAME_LENGTH, VctrlError};
use std::path::Path;

/// Validates a generic name.
///
/// # Errors
///
/// Returns [`VctrlError::InvalidName`] if the name is empty, exceeds the maximum
/// allowed length, or contains control characters.
pub fn validate_name(name: &str) -> Result<(), VctrlError> {
    if name.is_empty() {
        return Err(VctrlError::InvalidName("name is empty".into()));
    }
    let max_len = usize::try_from(MAX_NAME_LENGTH).unwrap_or(usize::MAX);
    if name.len() > max_len {
        return Err(VctrlError::InvalidName(format!(
            "name exceeds maximum length {MAX_NAME_LENGTH}: '{name}'"
        )));
    }
    if name.chars().any(char::is_control) {
        return Err(VctrlError::InvalidName(format!(
            "name contains control characters: '{name}'"
        )));
    }
    Ok(())
}

/// Validates a reference name (e.g., branch or tag) strictly according to Git rules.
///
/// # Errors
///
/// Returns [`VctrlError::InvalidName`] if the name fails basic name validation
/// or contains forbidden characters or patterns.
pub fn validate_ref_name(name: &str) -> Result<(), VctrlError> {
    validate_name(name)?;
    if name.contains("..")
        || name.contains('~')
        || name.contains('^')
        || name.contains(':')
        || name.contains('?')
        || name.contains('*')
        || name.contains('[')
        || name.contains('\\')
        || name.contains(' ')
        || name.contains("@{")
        || name.contains("//")
        || name.starts_with('.')
        || name.starts_with('/')
        || name.ends_with('/')
        || name.ends_with('.')
        || name.contains('<')
        || name.contains('>')
        || name.contains('|')
        || name.contains('"')
        || Path::new(name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("lock"))
    {
        return Err(VctrlError::InvalidName(format!(
            "invalid ref name: '{name}'"
        )));
    }
    Ok(())
}

/// Validates a tree entry name strictly.
///
/// # Errors
///
/// Returns [`VctrlError::InvalidName`] if the name fails basic name validation
/// or contains forbidden path characters or names.
pub fn validate_tree_entry_name(name: &str) -> Result<(), VctrlError> {
    validate_name(name)?;
    if name.contains('/') || name.contains('\\') || name == "." || name == ".." {
        return Err(VctrlError::InvalidName(format!(
            "tree entry name contains forbidden path characters or names: '{name}'"
        )));
    }
    Ok(())
}
