//! Core data types and validation helpers.

pub mod core;

use crate::constants::MAX_NAME_LENGTH;
use crate::errors::VctrlError;

// Re-export semua item publik dari core
pub use core::*;

// Re-export submodule `core` agar path seperti `types::blob::Blob` tetap valid
pub use core::{blob, commit, delta, hash, merge, reflog, tag, tree, user_id};

/// Returns `true` if the string contains any Unicode control character.
fn contains_control_character(s: &str) -> bool {
    s.chars().any(|c| c.is_control())
}

/// Validates a generic name.
///
/// # Errors
///
/// Returns [`VctrlError::InvalidName`] if the name is empty, too long, or contains control characters.
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

/// Validates a tree entry name.
///
/// # Errors
///
/// Returns [`VctrlError::InvalidName`] if the name is invalid or contains path separators or reserved names.
pub(crate) fn validate_tree_entry_name(name: &str) -> Result<(), VctrlError> {
    validate_name(name)?;
    if name.contains('/') || name == "." || name == ".." {
        return Err(VctrlError::InvalidName(format!(
            "tree entry name contains forbidden path characters or names: '{name}'"
        )));
    }
    Ok(())
}
