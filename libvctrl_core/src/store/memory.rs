//! In‑memory object store – the simplest [`ObjectStore`] implementation.
//!
//! [`MemoryStore`] implements [`ObjectStore`] using a `HashMap`. It is
//! useful for testing, prototyping, and as a reference for building
//! real backends.
//!
//! # Design
//!
//! The store is **content‑addressable**: objects are stored by their
//! [`Hash`] and retrieved by the same hash. The hash is never verified
//! against the content – that responsibility lies with the caller.
//!
//! # Performance
//!
//! - **put**: O(1) average – inserts into the `HashMap`.
//! - **get**: O(1) average – looks up and clones the value.
//! - **delete**: O(1) average – removes from the `HashMap`.
//! - **exists**: O(1) average – checks for key presence.
//!
//! The `Clone` in `get` means you receive an independent copy of the data.
//! This is safe but can be expensive for large objects. A production
//! backend might return a reference or use a copy‑on‑write strategy.
//!
//! # Memory usage
//!
//! The store holds a copy of every object. There is no garbage collection
//! or deduplication beyond what the caller does (e.g., storing the same
//! data twice under different hashes will consume twice the memory).
//!
//! # Idempotency
//!
//! Storing the same `(hash, data)` pair multiple times is safe and does
//! not fail. It simply overwrites the previous entry. This matches the
//! behaviour required by the trait contract.
//!
//! # Examples
//!
//! ```rust
//! use libvctrl_core::store::MemoryStore;
//! use libvctrl_handler::{Hash, ObjectStore, HASH_LENGTH};
//!
//! let mut store = MemoryStore::new();
//! let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
//!
//! // Store data
//! store.put(&hash, b"hello").unwrap();
//! assert!(store.exists(&hash).unwrap());
//!
//! // Retrieve data
//! let data = store.get(&hash).unwrap();
//! assert_eq!(data, b"hello");
//!
//! // Delete data
//! store.delete(&hash).unwrap();
//! assert!(!store.exists(&hash).unwrap());
//! ```

use libvctrl_handler::{Hash, ObjectStore, VctrlError};
use std::collections::HashMap;

/// An in‑memory object store backed by a [`HashMap`].
///
/// # Characteristics
/// - **Fast**: all operations are O(1) average.
/// - **Not thread‑safe**: wrap in `Arc<Mutex<…>>` for shared access.
/// - **Not persistent**: data is lost when the store is dropped.
///
/// # Examples
/// ```
/// use libvctrl_core::store::MemoryStore;
/// use libvctrl_handler::{Hash, ObjectStore, HASH_LENGTH};
///
/// let mut store = MemoryStore::new();
/// let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
/// store.put(&hash, b"hello").unwrap();
/// assert!(store.exists(&hash).unwrap());
/// assert_eq!(store.get(&hash).unwrap(), b"hello");
/// ```
#[derive(Debug, Default)]
pub struct MemoryStore {
    objects: HashMap<Hash, Vec<u8>>,
}

impl MemoryStore {
    /// Creates an empty store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
        }
    }
}

impl ObjectStore for MemoryStore {
    /// Store raw data under the given hash.
    ///
    /// # Errors
    /// This implementation never fails (it always returns `Ok(())`).
    fn put(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError> {
        let _ = self.objects.insert(*hash, data.to_vec());
        Ok(())
    }

    /// Retrieve raw data by hash.
    ///
    /// # Errors
    /// Returns [`VctrlError::ObjectNotFound`] if the hash is not in the store.
    fn get(&self, hash: &Hash) -> Result<Vec<u8>, VctrlError> {
        self.objects
            .get(hash)
            .cloned()
            .ok_or(VctrlError::ObjectNotFound(*hash))
    }

    /// Delete the object identified by `hash`.
    ///
    /// Deleting a non‑existent object succeeds silently.
    ///
    /// # Errors
    /// This implementation never fails (it always returns `Ok(())`).
    fn delete(&mut self, hash: &Hash) -> Result<(), VctrlError> {
        let _ = self.objects.remove(hash);
        Ok(())
    }

    /// Check whether an object exists under the given hash.
    ///
    /// # Errors
    /// This implementation never fails (it always returns `Ok(bool)`).
    fn exists(&self, hash: &Hash) -> Result<bool, VctrlError> {
        Ok(self.objects.contains_key(hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libvctrl_handler::{HASH_LENGTH, Hash, ObjectStore};

    fn dummy_hash(byte: u8) -> Hash {
        let mut arr = [byte; HASH_LENGTH];
        arr[0] = byte; // ensure uniqueness
        Hash::from_bytes(&arr).unwrap()
    }

    #[test]
    fn put_and_get() {
        let mut store = MemoryStore::new();
        let hash = dummy_hash(1);
        let data = b"hello world";
        store.put(&hash, data).unwrap();
        assert!(store.exists(&hash).unwrap());
        assert_eq!(store.get(&hash).unwrap(), data);
    }

    #[test]
    fn get_non_existent_returns_error() {
        let store = MemoryStore::new();
        let hash = dummy_hash(2);
        assert!(store.get(&hash).is_err());
        assert!(!store.exists(&hash).unwrap());
    }

    #[test]
    fn delete_existing_object() {
        let mut store = MemoryStore::new();
        let hash = dummy_hash(3);
        store.put(&hash, b"data").unwrap();
        store.delete(&hash).unwrap();
        assert!(!store.exists(&hash).unwrap());
    }

    #[test]
    fn delete_non_existent_is_noop() {
        let mut store = MemoryStore::new();
        let hash = dummy_hash(4);
        assert!(store.delete(&hash).is_ok()); // must not panic
    }

    #[test]
    fn put_overwrites() {
        let mut store = MemoryStore::new();
        let hash = dummy_hash(5);
        store.put(&hash, b"old").unwrap();
        store.put(&hash, b"new").unwrap();
        assert_eq!(store.get(&hash).unwrap(), b"new");
    }
}
