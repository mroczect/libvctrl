//! In-memory reference storage backend for `libvctrl_core`.
//!
//! # Purpose
//! This module provides the [`MemoryRefStore`], a concrete implementation of the
//! [`RefStore`](libvctrl_handler::RefStore) trait. It uses a standard
//! `HashMap` to map human-readable reference names (e.g., "HEAD",
//! "refs/heads/main") to cryptographic [`Hash`]es.
//!
//! # Design rationale
//! - **Ephemeral and Mutable**: Unlike objects, which are immutable and
//!   content-addressed, references are frequently updated (e.g., when a new
//!   commit is made). This store provides a fast, ephemeral location for these
//!   mutations in RAM.
//! - **Separation from ObjectStore**: Keeping references in a separate store
//!   allows backends to optimize their storage layouts. For example, an object
//!   store might be content-addressable and read-only, while a ref store needs
//!   to support arbitrary name updates.
//! - **Input Validation**: The `set_ref` method enforces name length constraints
//!   ([`MAX_NAME_LENGTH`](libvctrl_handler::MAX_NAME_LENGTH)) to prevent
//!   resource exhaustion and ensure compatibility with filesystem-based backends.
//!
//! # Internal mechanism
//! The store maps a `String` to a 64-byte [`Hash`](libvctrl_handler::Hash).
//! Lookups, insertions, and deletions are average O(1) operations. The `Hash`
//! is `Copy`, so retrieving a reference returns a cheap stack copy rather than
//! a heap allocation.

use libvctrl_handler::{Hash, MAX_NAME_LENGTH, RefStore, VctrlError};
use std::collections::HashMap;

/// An in-memory implementation of the [`RefStore`](libvctrl_handler::RefStore) trait.
///
/// # Purpose
/// Stores version control references in a `HashMap` residing in RAM. This
/// backend is primarily intended for testing, ephemeral operations, and
/// in-memory state management.
///
/// # Design rationale
/// This struct derives [`Default`] to allow easy instantiation. The internal
/// `HashMap` is kept private to ensure that all modifications go through the
/// `RefStore` trait implementation, preserving the integrity of the interface
/// and ensuring name validation is always enforced.
///
/// # Examples
///
/// Setting and retrieving a branch reference:
///
/// ```
/// use libvctrl_core::store::MemoryRefStore;
/// use libvctrl_handler::{Hash, RefStore};
///
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
    /// Creates a new, empty `MemoryRefStore`.
    ///
    /// # Design rationale
    /// This is a standard constructor that initializes an empty `HashMap`. It
    /// is functionally equivalent to `MemoryRefStore::default()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::store::MemoryRefStore;
    /// use libvctrl_handler::{Hash, RefStore};
    ///
    /// let store = MemoryRefStore::new();
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// assert!(store.get_ref("HEAD").is_err());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            refs: HashMap::new(),
        }
    }
}

impl RefStore for MemoryRefStore {
    /// Sets or updates a named reference to point to a specific hash.
    ///
    /// # Design rationale
    /// This method enforces name validation by rejecting empty names or names
    /// exceeding [`MAX_NAME_LENGTH`](libvctrl_handler::MAX_NAME_LENGTH). If a
    /// reference with the same name already exists, its target hash is
    /// overwritten.
    ///
    /// # Errors
    /// Returns [`VctrlError::InvalidName`](libvctrl_handler::VctrlError::InvalidName)
    /// if the name is empty or exceeds the maximum allowed length.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::store::MemoryRefStore;
    /// use libvctrl_handler::{Hash, RefStore};
    ///
    /// let mut store = MemoryRefStore::new();
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// store.set_ref("HEAD", &hash).unwrap();
    /// ```
    fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError> {
        if name.is_empty() || name.len() > MAX_NAME_LENGTH {
            return Err(VctrlError::InvalidName(name.into()));
        }
        let _ = self.refs.insert(name.to_string(), *hash);
        Ok(())
    }

    /// Retrieves the hash a named reference points to.
    ///
    /// # Design rationale
    /// Returns a copied `Hash` value (which is `Copy`) rather than a reference,
    /// allowing the caller to use it without borrowing from the store.
    ///
    /// # Errors
    /// Returns [`VctrlError::RefNotFound`](libvctrl_handler::VctrlError::RefNotFound)
    /// if the reference does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::store::MemoryRefStore;
    /// use libvctrl_handler::{Hash, RefStore};
    ///
    /// let mut store = MemoryRefStore::new();
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// store.set_ref("HEAD", &hash).unwrap();
    /// assert_eq!(store.get_ref("HEAD").unwrap(), hash);
    /// ```
    fn get_ref(&self, name: &str) -> Result<Hash, VctrlError> {
        self.refs
            .get(name)
            .copied()
            .ok_or_else(|| VctrlError::RefNotFound(name.into()))
    }

    /// Deletes a named reference.
    ///
    /// # Design rationale
    /// This operation is idempotent. If the reference does not exist, the method
    /// silently succeeds without returning an error, simplifying cleanup logic.
    ///
    /// # Errors
    /// This implementation is infallible, but returns a `Result` to satisfy the
    /// trait interface.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::store::MemoryRefStore;
    /// use libvctrl_handler::{Hash, RefStore};
    ///
    /// let mut store = MemoryRefStore::new();
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// store.set_ref("HEAD", &hash).unwrap();
    /// store.delete_ref("HEAD").unwrap();
    /// assert!(store.get_ref("HEAD").is_err());
    /// ```
    fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError> {
        let _ = self.refs.remove(name);
        Ok(())
    }

    /// Lists all reference names currently stored.
    ///
    /// # Design rationale
    /// Collects all keys from the internal `HashMap` into a `Vec<String>`. Note
    /// that because `HashMap` iteration order is non-deterministic, the order
    /// of the resulting vector is not guaranteed.
    ///
    /// # Errors
    /// This implementation is infallible, but returns a `Result` to satisfy the
    /// trait interface.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::store::MemoryRefStore;
    /// use libvctrl_handler::{Hash, RefStore};
    ///
    /// let mut store = MemoryRefStore::new();
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// store.set_ref("a", &hash).unwrap();
    /// store.set_ref("b", &hash).unwrap();
    ///
    /// let mut refs = store.list_refs().unwrap();
    /// refs.sort(); // Sort for deterministic testing
    /// assert_eq!(refs, vec!["a".to_string(), "b".to_string()]);
    /// ```
    fn list_refs(&self) -> Result<Vec<String>, VctrlError> {
        Ok(self.refs.keys().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libvctrl_handler::HASH_LENGTH;

    fn dummy_hash() -> Hash {
        Hash::from_bytes(&[0xAB; HASH_LENGTH]).unwrap()
    }

    #[test]
    fn set_and_get_ref() {
        let mut store = MemoryRefStore::new();
        let hash = dummy_hash();
        store.set_ref("HEAD", &hash).unwrap();
        assert_eq!(store.get_ref("HEAD").unwrap(), hash);
    }

    #[test]
    fn get_non_existent_ref() {
        let store = MemoryRefStore::new();
        assert!(store.get_ref("HEAD").is_err());
    }

    #[test]
    fn delete_ref() {
        let mut store = MemoryRefStore::new();
        store.set_ref("refs/heads/main", &dummy_hash()).unwrap();
        store.delete_ref("refs/heads/main").unwrap();
        assert!(store.get_ref("refs/heads/main").is_err());
    }

    #[test]
    fn delete_non_existent_is_noop() {
        let mut store = MemoryRefStore::new();
        assert!(store.delete_ref("nope").is_ok());
    }

    #[test]
    fn set_ref_with_empty_name_fails() {
        let mut store = MemoryRefStore::new();
        assert!(store.set_ref("", &dummy_hash()).is_err());
    }

    #[test]
    fn set_ref_with_too_long_name_fails() {
        let mut store = MemoryRefStore::new();
        let long_name = "a".repeat(MAX_NAME_LENGTH + 1);
        assert!(store.set_ref(&long_name, &dummy_hash()).is_err());
    }

    #[test]
    fn list_refs() {
        let mut store = MemoryRefStore::new();
        store.set_ref("a", &dummy_hash()).unwrap();
        store.set_ref("b", &dummy_hash()).unwrap();
        let list = store.list_refs().unwrap();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&"a".to_string()));
        assert!(list.contains(&"b".to_string()));
    }

    #[test]
    fn overwrite_ref() {
        let mut store = MemoryRefStore::new();
        let hash1 = dummy_hash();
        let mut hash2_arr = [0xCD; HASH_LENGTH];
        hash2_arr[0] = 0xCD;
        let hash2 = Hash::from_bytes(&hash2_arr).unwrap();
        store.set_ref("HEAD", &hash1).unwrap();
        store.set_ref("HEAD", &hash2).unwrap();
        assert_eq!(store.get_ref("HEAD").unwrap(), hash2);
    }
}
