pub mod core;

use crate::constants::MAX_NAME_LENGTH;
use crate::errors::VctrlError;

pub use core::*;

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
    Ok(())
}

pub(crate) fn validate_tree_entry_name(name: &str) -> Result<(), VctrlError> {
    validate_name(name)?;
    if name.contains('/') || name == "." || name == ".." {
        return Err(VctrlError::InvalidName(format!(
            "tree entry name contains forbidden path characters or names: '{name}'"
        )));
    }
    Ok(())
}
