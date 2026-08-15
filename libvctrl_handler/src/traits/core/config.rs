//! Configuration store trait.

use crate::VctrlError;

/// A trait for reading configuration values.
pub trait ConfigStore {
    /// Returns the string value for the given section and key.
    ///
    /// Returns `Ok(None)` if the key is not found.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the configuration cannot be read.
    fn get_string(&self, section: &str, key: &str) -> Result<Option<String>, VctrlError>;
}
