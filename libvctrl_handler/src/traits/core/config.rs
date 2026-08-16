use crate::errors::VctrlError;

/// A trait for reading and writing configuration values.
pub trait ConfigStore: Send + Sync {
    /// Returns the string value for the given section and key.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the configuration cannot be read.
    fn get_string(&self, section: &str, key: &str) -> Result<Option<String>, VctrlError>;

    /// Sets the string value for the given section and key.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the configuration cannot be written.
    fn set_string(&mut self, section: &str, key: &str, value: &str) -> Result<(), VctrlError>;

    /// Returns the boolean value for the given section and key.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the configuration cannot be read or is not a boolean.
    fn get_bool(&self, section: &str, key: &str) -> Result<Option<bool>, VctrlError>;

    /// Sets the boolean value for the given section and key.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the configuration cannot be written.
    fn set_bool(&mut self, section: &str, key: &str, value: bool) -> Result<(), VctrlError>;

    /// Removes a key from the configuration.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the configuration cannot be modified.
    fn remove(&mut self, section: &str, key: &str) -> Result<(), VctrlError>;

    /// Checks if a key exists in the configuration.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the configuration cannot be read.
    fn exists(&self, section: &str, key: &str) -> Result<bool, VctrlError>;
}
