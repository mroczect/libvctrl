//! # In-Memory Reference Store
//!
//! This module provides [`MemoryRefStore`], a lightweight implementation of the
//! [`RefStore`](libvctrl_handler::RefStore) trait backed by a
//! [`std::collections::HashMap`].
//!
//! The store is intended for testing, prototyping, and scenarios where
//! persistence is not required. It stores references in memory only and loses
//! all data when dropped.
//!
//! ## Why this exists
//!
//! The [`RefStore`](libvctrl_handler::RefStore) trait defines the contract for
//! managing named references such as branches and tags. A concrete in-memory
//! implementation is essential for unit tests, examples, and as a reference
//! backend. It also demonstrates the expected behavior of the trait without
//! any disk or network dependencies.
//!
//! ## How it works
//!
//! References are stored in a private `HashMap<String, Hash>`. The `set_ref`
//! method validates the reference name using
//! [`validate_ref_name`](libvctrl_handler::validate_ref_name) before inserting.
//! The `list_refs` method collects and sorts all keys to provide deterministic
//! iteration order.

use libvctrl_handler::{Hash, RefStore, VctrlError};
use std::collections::HashMap;

/// An in-memory implementation of [`RefStore`].
///
/// `MemoryRefStore` stores named references such as branches and tags in a
/// `HashMap<String, Hash>`. It is suitable for ephemeral use cases and testing.
///
/// # Why this struct exists
///
/// The [`RefStore`] trait requires an implementation to be useful. This struct
/// provides a minimal, safe, and deterministic reference store that can be
/// embedded in applications or used as a baseline for tests.
///
/// # How it works
///
/// Internally, references are keyed by name and mapped to their target
/// [`Hash`]. The store validates names on insertion and returns errors when
/// lookups fail.
///
/// # Examples
///
/// ```
/// # use libvctrl_core::store::MemoryRefStore;
/// # use libvctrl_handler::{Hash, RefStore};
/// let mut store = MemoryRefStore::new();
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
///
/// store.set_ref("refs/heads/main", &hash).unwrap();
/// assert_eq!(store.get_ref("refs/heads/main").unwrap(), hash);
/// ```
#[derive(Debug, Default)]
pub struct MemoryRefStore {
    refs: HashMap<String, Hash>,
}

impl MemoryRefStore {
    /// Creates a new empty `MemoryRefStore`.
    ///
    /// The store contains no references initially.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::store::MemoryRefStore;
    /// use libvctrl_handler::RefStore;
    /// let store = MemoryRefStore::new();
    /// assert!(store.list_refs().unwrap().next().is_none());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            refs: HashMap::new(),
        }
    }
}

impl RefStore for MemoryRefStore {
    type RefsIterator = std::vec::IntoIter<Result<String, VctrlError>>;

    /// Sets or updates a reference.
    ///
    /// The reference name is validated before insertion. If the name already
    /// exists, its target hash is replaced.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if `name` is invalid according to
    /// [`validate_ref_name`](libvctrl_handler::validate_ref_name).
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_core::store::MemoryRefStore;
    /// # use libvctrl_handler::{Hash, RefStore};
    /// let mut store = MemoryRefStore::new();
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    ///
    /// store.set_ref("refs/heads/main", &hash).unwrap();
    /// assert!(store.get_ref("refs/heads/main").is_ok());
    /// ```
    fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError> {
        libvctrl_handler::validate_ref_name(name)?;
        let _ = self.refs.insert(name.to_string(), *hash);
        Ok(())
    }

    /// Retrieves the target hash for a reference.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::RefNotFound`] if no reference with the given name
    /// exists.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_core::store::MemoryRefStore;
    /// # use libvctrl_handler::{Hash, RefStore};
    /// let mut store = MemoryRefStore::new();
    /// let hash = Hash::from_bytes(&[1u8; 64]).unwrap();
    /// store.set_ref("refs/heads/main", &hash).unwrap();
    ///
    /// assert_eq!(store.get_ref("refs/heads/main").unwrap(), hash);
    /// ```
    fn get_ref(&self, name: &str) -> Result<Hash, VctrlError> {
        self.refs
            .get(name)
            .copied()
            .ok_or_else(|| VctrlError::RefNotFound(name.into()))
    }

    /// Deletes a reference.
    ///
    /// If the reference does not exist, this method does nothing and returns
    /// `Ok(())`.
    ///
    /// # Errors
    ///
    /// This method currently cannot fail; it always returns `Ok(())`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_core::store::MemoryRefStore;
    /// # use libvctrl_handler::{Hash, RefStore};
    /// let mut store = MemoryRefStore::new();
    /// let hash = Hash::from_bytes(&[2u8; 64]).unwrap();
    /// store.set_ref("refs/heads/temp", &hash).unwrap();
    ///
    /// store.delete_ref("refs/heads/temp").unwrap();
    /// assert!(store.get_ref("refs/heads/temp").is_err());
    /// ```
    fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError> {
        let _ = self.refs.remove(name);
        Ok(())
    }

    /// Lists all reference names in sorted order.
    ///
    /// The returned iterator yields `Result<String, VctrlError>`. Sorting
    /// ensures deterministic output, which is important for tests and
    /// reproducibility.
    ///
    /// # Errors
    ///
    /// This method currently cannot fail; it always returns `Ok(iterator)`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_core::store::MemoryRefStore;
    /// # use libvctrl_handler::{Hash, RefStore};
    /// let mut store = MemoryRefStore::new();
    /// let hash = Hash::from_bytes(&[3u8; 64]).unwrap();
    /// store.set_ref("refs/heads/b", &hash).unwrap();
    /// store.set_ref("refs/heads/a", &hash).unwrap();
    ///
    /// let names: Vec<String> = store
    ///     .list_refs()
    ///     .unwrap()
    ///     .map(|r| r.unwrap())
    ///     .collect();
    /// assert_eq!(names, vec!["refs/heads/a".to_owned(), "refs/heads/b".to_owned()]);
    /// ```
    fn list_refs(&self) -> Result<Self::RefsIterator, VctrlError> {
        let mut names: Vec<String> = self.refs.keys().cloned().collect();
        names.sort();
        Ok(names.into_iter().map(Ok).collect::<Vec<_>>().into_iter())
    }
}
