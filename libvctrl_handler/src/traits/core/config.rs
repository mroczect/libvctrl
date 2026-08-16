//! Configuration store trait.
//!
//! # Architecture
//! This module defines the abstract contract for reading and writing repository
//! configuration settings (e.g., `.git/config`). By abstracting this into a trait,
//! the crate decouples the core engine from the underlying storage mechanism,
//! allowing consumers to use INI files, databases, or in-memory hash maps.
//!
//! # Design Rationale: `Option` vs `Result`
//! Configuration is inherently sparse. A missing key is often a valid state indicating
//! that a default value should be used, not an exceptional error. Therefore, read
//! operations return `Option<T>`. An `Err(VctrlError)` is reserved strictly for
//! I/O failures or parsing corruption, ensuring a clear distinction between
//! "key not set" and "failed to read configuration".

use crate::errors::VctrlError;

/// A trait for reading and writing configuration values.
///
/// # Why this exists
/// Provides a unified, type-safe interface for managing repository settings. Git
/// configurations are segmented by sections (e.g., `user`, `core`) and keys.
/// This trait enforces that structure, preventing malformed configuration access
/// and allowing backend-agnostic validation.
///
/// # Design Rationale: Thread Safety
/// The trait requires `Send + Sync`. Configuration is frequently read by multiple
/// concurrent operations (e.g., checking commit hooks, resolving user identities)
/// but rarely written. This trait design allows implementors to use `RwLock`
/// internally or rely on immutable snapshots, enabling safe parallel reads across
/// threads without locking the entire repository state.
///
/// # Examples
///
/// Implementing the trait for a mock in-memory store:
///
/// ```
/// # use libvctrl_handler::traits::core::config::ConfigStore;
/// # use libvctrl_handler::VctrlError;
/// # use std::collections::HashMap;
/// #
/// #[derive(Default)]
/// struct MockConfig {
///     data: HashMap<String, String>,
/// }
///
/// impl ConfigStore for MockConfig {
///     fn get_string(&self, section: &str, key: &str) -> Result<Option<String>, VctrlError> {
///         let full_key = format!("{section}.{key}");
///         Ok(self.data.get(&full_key).cloned())
///     }
///
///     fn set_string(&mut self, section: &str, key: &str, value: &str) -> Result<(), VctrlError> {
///         let full_key = format!("{section}.{key}");
///         self.data.insert(full_key, value.to_string());
///         Ok(())
///     }
///
///     fn get_bool(&self, section: &str, key: &str) -> Result<Option<bool>, VctrlError> {
///         Ok(self.get_string(section, key)?.map(|v| v == "true"))
///     }
///
///     fn set_bool(&mut self, section: &str, key: &str, value: bool) -> Result<(), VctrlError> {
///         self.set_string(section, key, if value { "true" } else { "false" })
///     }
///
///     fn remove(&mut self, section: &str, key: &str) -> Result<(), VctrlError> {
///         let full_key = format!("{section}.{key}");
///         self.data.remove(&full_key);
///         Ok(())
///     }
///
///     fn exists(&self, section: &str, key: &str) -> Result<bool, VctrlError> {
///         let full_key = format!("{section}.{key}");
///         Ok(self.data.contains_key(&full_key))
///     }
/// }
///
/// let mut cfg = MockConfig::default();
/// cfg.set_string("user", "name", "Alice")?;
/// assert_eq!(cfg.get_string("user", "name")?, Some("Alice".to_string()));
/// # Ok::<(), VctrlError>(())
/// ```
pub trait ConfigStore: Send + Sync {
    /// Returns the string value for the given section and key.
    ///
    /// # How it works
    /// Looks up the configuration value in the specified section. If the section
    /// or key does not exist, it returns `Ok(None)` rather than an error, allowing
    /// the caller to fall back to default values gracefully.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the configuration cannot be read (e.g., due to
    /// an I/O failure or corrupted configuration file).
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::config::ConfigStore;
    /// # use libvctrl_handler::VctrlError;
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct MockConfig { data: HashMap<String, String> }
    /// # impl ConfigStore for MockConfig {
    /// #     fn get_string(&self, s: &str, k: &str) -> Result<Option<String>, VctrlError> { Ok(self.data.get(&format!("{s}.{k}")).cloned()) }
    /// #     fn set_string(&mut self, s: &str, k: &str, v: &str) -> Result<(), VctrlError> { self.data.insert(format!("{s}.{k}"), v.to_string()); Ok(()) }
    /// #     fn get_bool(&self, s: &str, k: &str) -> Result<Option<bool>, VctrlError> { Ok(self.get_string(s, k)?.map(|v| v == "true")) }
    /// #     fn set_bool(&mut self, s: &str, k: &str, v: bool) -> Result<(), VctrlError> { self.set_string(s, k, if v { "true" } else { "false" }) }
    /// #     fn remove(&mut self, s: &str, k: &str) -> Result<(), VctrlError> { self.data.remove(&format!("{s}.{k}")); Ok(()) }
    /// #     fn exists(&self, s: &str, k: &str) -> Result<bool, VctrlError> { Ok(self.data.contains_key(&format!("{s}.{k}"))) }
    /// # }
    /// let mut cfg = MockConfig::default();
    /// cfg.set_string("core", "editor", "vim")?;
    /// assert_eq!(cfg.get_string("core", "editor")?, Some("vim".to_string()));
    /// assert_eq!(cfg.get_string("core", "missing")?, None);
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn get_string(&self, section: &str, key: &str) -> Result<Option<String>, VctrlError>;

    /// Sets the string value for the given section and key.
    ///
    /// # How it works
    /// Requires `&mut self`, enforcing exclusive access for write operations. This
    /// ensures that no other thread can read a partially written configuration state,
    /// maintaining atomicity at the trait level. Implementors are responsible for
    /// persisting this change to the underlying storage medium.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the configuration cannot be written (e.g., due to
    /// insufficient permissions or disk full).
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::config::ConfigStore;
    /// # use libvctrl_handler::VctrlError;
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct MockConfig { data: HashMap<String, String> }
    /// # impl ConfigStore for MockConfig {
    /// #     fn get_string(&self, s: &str, k: &str) -> Result<Option<String>, VctrlError> { Ok(self.data.get(&format!("{s}.{k}")).cloned()) }
    /// #     fn set_string(&mut self, s: &str, k: &str, v: &str) -> Result<(), VctrlError> { self.data.insert(format!("{s}.{k}"), v.to_string()); Ok(()) }
    /// #     fn get_bool(&self, s: &str, k: &str) -> Result<Option<bool>, VctrlError> { Ok(self.get_string(s, k)?.map(|v| v == "true")) }
    /// #     fn set_bool(&mut self, s: &str, k: &str, v: bool) -> Result<(), VctrlError> { self.set_string(s, k, if v { "true" } else { "false" }) }
    /// #     fn remove(&mut self, s: &str, k: &str) -> Result<(), VctrlError> { self.data.remove(&format!("{s}.{k}")); Ok(()) }
    /// #     fn exists(&self, s: &str, k: &str) -> Result<bool, VctrlError> { Ok(self.data.contains_key(&format!("{s}.{k}"))) }
    /// # }
    /// let mut cfg = MockConfig::default();
    /// cfg.set_string("user", "email", "test@example.com")?;
    /// assert!(cfg.exists("user", "email")?);
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn set_string(&mut self, section: &str, key: &str, value: &str) -> Result<(), VctrlError>;

    /// Returns the boolean value for the given section and key.
    ///
    /// # How it works
    /// Retrieves the string representation and attempts to parse it as a boolean.
    /// If the key exists but is not a valid boolean (e.g., "yes", "1", "true"),
    /// the implementor should return a [`VctrlError::SerializationError`] or similar,
    /// as this indicates a corrupted or malformed configuration.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the configuration cannot be read or is not a boolean.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::config::ConfigStore;
    /// # use libvctrl_handler::VctrlError;
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct MockConfig { data: HashMap<String, String> }
    /// # impl ConfigStore for MockConfig {
    /// #     fn get_string(&self, s: &str, k: &str) -> Result<Option<String>, VctrlError> { Ok(self.data.get(&format!("{s}.{k}")).cloned()) }
    /// #     fn set_string(&mut self, s: &str, k: &str, v: &str) -> Result<(), VctrlError> { self.data.insert(format!("{s}.{k}"), v.to_string()); Ok(()) }
    /// #     fn get_bool(&self, s: &str, k: &str) -> Result<Option<bool>, VctrlError> { Ok(self.get_string(s, k)?.map(|v| v == "true")) }
    /// #     fn set_bool(&mut self, s: &str, k: &str, v: bool) -> Result<(), VctrlError> { self.set_string(s, k, if v { "true" } else { "false" }) }
    /// #     fn remove(&mut self, s: &str, k: &str) -> Result<(), VctrlError> { self.data.remove(&format!("{s}.{k}")); Ok(()) }
    /// #     fn exists(&self, s: &str, k: &str) -> Result<bool, VctrlError> { Ok(self.data.contains_key(&format!("{s}.{k}"))) }
    /// # }
    /// let mut cfg = MockConfig::default();
    /// cfg.set_bool("core", "bare", true)?;
    /// assert_eq!(cfg.get_bool("core", "bare")?, Some(true));
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn get_bool(&self, section: &str, key: &str) -> Result<Option<bool>, VctrlError>;

    /// Sets the boolean value for the given section and key.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the configuration cannot be written.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::config::ConfigStore;
    /// # use libvctrl_handler::VctrlError;
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct MockConfig { data: HashMap<String, String> }
    /// # impl ConfigStore for MockConfig {
    /// #     fn get_string(&self, s: &str, k: &str) -> Result<Option<String>, VctrlError> { Ok(self.data.get(&format!("{s}.{k}")).cloned()) }
    /// #     fn set_string(&mut self, s: &str, k: &str, v: &str) -> Result<(), VctrlError> { self.data.insert(format!("{s}.{k}"), v.to_string()); Ok(()) }
    /// #     fn get_bool(&self, s: &str, k: &str) -> Result<Option<bool>, VctrlError> { Ok(self.get_string(s, k)?.map(|v| v == "true")) }
    /// #     fn set_bool(&mut self, s: &str, k: &str, v: bool) -> Result<(), VctrlError> { self.set_string(s, k, if v { "true" } else { "false" }) }
    /// #     fn remove(&mut self, s: &str, k: &str) -> Result<(), VctrlError> { self.data.remove(&format!("{s}.{k}")); Ok(()) }
    /// #     fn exists(&self, s: &str, k: &str) -> Result<bool, VctrlError> { Ok(self.data.contains_key(&format!("{s}.{k}"))) }
    /// # }
    /// let mut cfg = MockConfig::default();
    /// cfg.set_bool("core", "autocrlf", false)?;
    /// assert_eq!(cfg.get_string("core", "autocrlf")?, Some("false".to_string()));
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn set_bool(&mut self, section: &str, key: &str, value: bool) -> Result<(), VctrlError>;

    /// Removes a key from the configuration.
    ///
    /// # How it works
    /// Deletes the specified key within the given section. If the key or section
    /// does not exist, this operation is idempotent and returns `Ok(())`, ensuring
    /// that cleanup operations do not fail spuriously on missing data.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the configuration cannot be modified (e.g., due to
    /// file permission issues).
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::config::ConfigStore;
    /// # use libvctrl_handler::VctrlError;
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct MockConfig { data: HashMap<String, String> }
    /// # impl ConfigStore for MockConfig {
    /// #     fn get_string(&self, s: &str, k: &str) -> Result<Option<String>, VctrlError> { Ok(self.data.get(&format!("{s}.{k}")).cloned()) }
    /// #     fn set_string(&mut self, s: &str, k: &str, v: &str) -> Result<(), VctrlError> { self.data.insert(format!("{s}.{k}"), v.to_string()); Ok(()) }
    /// #     fn get_bool(&self, s: &str, k: &str) -> Result<Option<bool>, VctrlError> { Ok(self.get_string(s, k)?.map(|v| v == "true")) }
    /// #     fn set_bool(&mut self, s: &str, k: &str, v: bool) -> Result<(), VctrlError> { self.set_string(s, k, if v { "true" } else { "false" }) }
    /// #     fn remove(&mut self, s: &str, k: &str) -> Result<(), VctrlError> { self.data.remove(&format!("{s}.{k}")); Ok(()) }
    /// #     fn exists(&self, s: &str, k: &str) -> Result<bool, VctrlError> { Ok(self.data.contains_key(&format!("{s}.{k}"))) }
    /// # }
    /// let mut cfg = MockConfig::default();
    /// cfg.set_string("remote", "origin", "url")?;
    /// cfg.remove("remote", "origin")?;
    /// assert!(!cfg.exists("remote", "origin")?);
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn remove(&mut self, section: &str, key: &str) -> Result<(), VctrlError>;

    /// Checks if a key exists in the configuration.
    ///
    /// # How it works
    /// Performs a lightweight existence check without retrieving the value. This is
    /// useful for validating configuration prerequisites before attempting complex
    /// operations.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the configuration cannot be read.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::config::ConfigStore;
    /// # use libvctrl_handler::VctrlError;
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct MockConfig { data: HashMap<String, String> }
    /// # impl ConfigStore for MockConfig {
    /// #     fn get_string(&self, s: &str, k: &str) -> Result<Option<String>, VctrlError> { Ok(self.data.get(&format!("{s}.{k}")).cloned()) }
    /// #     fn set_string(&mut self, s: &str, k: &str, v: &str) -> Result<(), VctrlError> { self.data.insert(format!("{s}.{k}"), v.to_string()); Ok(()) }
    /// #     fn get_bool(&self, s: &str, k: &str) -> Result<Option<bool>, VctrlError> { Ok(self.get_string(s, k)?.map(|v| v == "true")) }
    /// #     fn set_bool(&mut self, s: &str, k: &str, v: bool) -> Result<(), VctrlError> { self.set_string(s, k, if v { "true" } else { "false" }) }
    /// #     fn remove(&mut self, s: &str, k: &str) -> Result<(), VctrlError> { self.data.remove(&format!("{s}.{k}")); Ok(()) }
    /// #     fn exists(&self, s: &str, k: &str) -> Result<bool, VctrlError> { Ok(self.data.contains_key(&format!("{s}.{k}"))) }
    /// # }
    /// let cfg = MockConfig::default();
    /// assert!(!cfg.exists("nonexistent", "key")?);
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn exists(&self, section: &str, key: &str) -> Result<bool, VctrlError>;
}
