pub mod core;

use crate::constants::MAX_NAME_LENGTH;
use crate::errors::VctrlError;
use std::path::Path;

pub use core::*;

fn contains_control_character(s: &str) -> bool {
    s.chars().any(char::is_control)
}

#[allow(clippy::cast_possible_truncation)]
pub(crate) fn validate_name(name: &str) -> Result<(), VctrlError> {
    if name.is_empty() {
        return Err(VctrlError::InvalidName("name is empty".into()));
    }
    if name.len() > MAX_NAME_LENGTH as usize {
        return Err(VctrlError::InvalidName(format!(
            "name exceeds maximum length {MAX_NAME_LENGTH}: '{name}'"
        )));
    }
    if contains_control_character(name) {
        return Err(VctrlError::InvalidName(format!(
            "name contains control characters: '{name}'"
        )));
    }
    Ok(())
}

pub(crate) fn validate_ref_name(name: &str) -> Result<(), VctrlError> {
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
        || name.starts_with('.')
        || name.starts_with('/')
        || name.ends_with('/')
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

pub(crate) fn validate_tree_entry_name(name: &str) -> Result<(), VctrlError> {
    validate_name(name)?;
    if name.contains('/') || name.contains('\\') || name == "." || name == ".." {
        return Err(VctrlError::InvalidName(format!(
            "tree entry name contains forbidden path characters or names: '{name}'"
        )));
    }
    Ok(())
}
