use std::path::Path;

use crate::constants::MAX_NAME_LENGTH;
use crate::errors::VctrlError;

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
    if name.bytes().any(|b| b.is_ascii_control()) {
        return Err(VctrlError::InvalidName(format!(
            "name contains control characters: '{name}'"
        )));
    }
    Ok(())
}

pub fn validate_ref_name(name: &str) -> Result<(), VctrlError> {
    validate_name(name)?;

    if name == "@" {
        return Err(VctrlError::InvalidName("ref name cannot be '@'".into()));
    }

    if name.starts_with('/') || name.ends_with('/') || name.contains("//") {
        return Err(VctrlError::InvalidName(format!(
            "invalid ref name: '{name}'"
        )));
    }

    for component in name.split('/') {
        if component.is_empty()
            || component.starts_with('.')
            || Path::new(component)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("lock"))
            || component.contains("..")
            || component.contains('~')
            || component.contains('^')
            || component.contains(':')
            || component.contains('?')
            || component.contains('*')
            || component.contains('[')
            || component.contains('\\')
            || component.contains(' ')
            || component.contains("@{")
            || component.contains('<')
            || component.contains('>')
            || component.contains('|')
            || component.contains('"')
        {
            return Err(VctrlError::InvalidName(format!(
                "invalid ref name: '{name}'"
            )));
        }
    }

    Ok(())
}

pub fn validate_tree_entry_name(name: &str) -> Result<(), VctrlError> {
    validate_name(name)?;
    if name.contains('/') || name.contains('\\') || name == "." || name == ".." {
        return Err(VctrlError::InvalidName(format!(
            "tree entry name contains forbidden path characters or names: '{name}'"
        )));
    }
    Ok(())
}
