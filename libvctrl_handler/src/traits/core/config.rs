//! Defines the `ConfigStore` trait for accessing configuration values.
//!
//! # Purpose
//!
//! The `ConfigStore` trait abstracts access to configuration variables,
//! such as user name, email, and diff settings. This allows different
//! backends (in-memory, file-based, gitconfig) to provide configuration
//! data without coupling the caller to a specific storage mechanism.
//!
//! # Why a separate module
//!
//! Configuration is a cross-cutting concern that must be swappable.
//! Keeping the trait in its own file follows the same pattern as other
//! core traits (`ObjectStore`, `RefStore`, etc.), enabling independent
//! implementations.
//!
//! # Examples
//!
//! A simple in-memory implementation:
//!
//! ```
//! use std::collections::HashMap;
//! use libvctrl_handler::{ConfigStore, VctrlError};
//!
//! struct MemoryConfig {
//!     values: HashMap<String, String>,
//! }
//!
//! impl ConfigStore for MemoryConfig {
//!     fn get_string(&self, section: &str, key: &str) -> Result<Option<String>, VctrlError> {
//!         let full_key = format!("{}.{}", section, key);
//!         Ok(self.values.get(&full_key).cloned())
//!     }
//! }
//!
//! let mut map = HashMap::new();
//! map.insert("user.name".to_string(), "Alice".to_string());
//! let config = MemoryConfig { values: map };
//!
//! assert_eq!(
//!     config.get_string("user", "name").unwrap(),
//!     Some("Alice".to_string())
//! );
//! assert_eq!(config.get_string("user", "email").unwrap(), None);
//! ```

use crate::VctrlError;

/// Trait for accessing configuration variables.
///
/// # Purpose
///
/// `ConfigStore` abstracts read access to configuration values, allowing
/// implementations to source data from memory, files, or system gitconfig.
/// It is primarily used to retrieve settings such as user identity and
/// diff preferences during porcelain command execution.
///
/// # Examples
///
/// A trivial implementation that returns `None` for every key:
///
/// ```
/// use libvctrl_handler::{ConfigStore, VctrlError};
///
/// struct EmptyConfig;
///
/// impl ConfigStore for EmptyConfig {
///     fn get_string(&self, _section: &str, _key: &str) -> Result<Option<String>, VctrlError> {
///         Ok(None)
///     }
/// }
///
/// let config = EmptyConfig;
/// assert_eq!(config.get_string("user", "name").unwrap(), None);
/// ```
///
/// # Errors
///
/// - [`VctrlError::Other`] if the underlying configuration backend fails.
pub trait ConfigStore {
    /// Returns the configuration value for the given section and key.
    ///
    /// Returns `Ok(None)` if the key does not exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration backend cannot be accessed.
    fn get_string(&self, section: &str, key: &str) -> Result<Option<String>, VctrlError>;
}
