//! In-memory object storage backend for `libvctrl_core`.
//!
//! # Purpose
//! This module provides the [`MemoryStore`], a concrete implementation of the
//! [`ObjectStore`](libvctrl_handler::ObjectStore) trait. It uses a standard
//! `HashMap` to store serialized version control objects in RAM.
//!
//! # Design rationale
//! - **Ephemeral Storage**: As a non-persistent store, it is ideal for unit
//!   testing, temporary caching, or short-lived sessions where disk or network
//!   I/O is unnecessary or undesirable.
//! - **Streaming Reads**: The `get` method returns a `Box<dyn Read>`, which
//!   wraps the data in a [`std::io::Cursor`]. This design allows the caller to
//!   stream the object's contents incrementally, which is crucial for large
//!   blobs that should not be loaded entirely into memory at once.
//! - **Idempotent Deletion**: The `delete` method acts as a no-op if the object
//!   does not exist, preventing errors during cleanup and simplifying caller
//!   logic.
//!
//! # Internal mechanism
//! The store maps a 64-byte [`Hash`](libvctrl_handler::Hash) to a `Vec<u8>`.
//! Data is copied on insertion (`put`) into a new owned buffer. On retrieval
//! (`get`), the internal `Vec<u8>` is cloned and wrapped in a
//! [`std::io::Cursor`]. This cloning ensures that the caller receives an
//! independent snapshot of the data and does not hold a lock on the store's
//! internal structures.

use libvctrl_handler::{Hash, ObjectStore, VctrlError};
use std::collections::HashMap;
use std::io::{Cursor, Read};

/// An in-memory implementation of the [`ObjectStore`](libvctrl_handler::ObjectStore) trait.
///
/// # Purpose
/// Stores version control objects in a `HashMap` residing in RAM. This backend
/// is primarily intended for testing and ephemeral operations.
///
/// # Design rationale
/// This struct derives [`Default`] to allow easy instantiation via
/// `MemoryStore::default()` or `MemoryStore::new()`. The internal `HashMap` is
/// kept private to ensure that all modifications go through the `ObjectStore`
/// trait implementation, preserving the integrity of the storage interface.
///
/// # Examples
///
/// Storing and retrieving a blob via streaming:
///
/// ```
/// use libvctrl_core::store::MemoryStore;
/// use libvctrl_handler::{Hash, ObjectStore};
/// use std::io::Read;
///
/// let mut store = MemoryStore::new();
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
///
/// store.put(&hash, b"my data").unwrap();
/// assert!(store.exists(&hash).unwrap());
///
/// let mut reader = store.get(&hash).unwrap();
/// let mut buf = Vec::new();
/// reader.read_to_end(&mut buf).unwrap();
///
/// assert_eq!(buf, b"my data");
/// ```
#[derive(Debug, Default)]
pub struct MemoryStore {
    objects: HashMap<Hash, Vec<u8>>,
}

impl MemoryStore {
    /// Creates a new, empty `MemoryStore`.
    ///
    /// # Design rationale
    /// This is a standard constructor that initializes an empty `HashMap`. It
    /// is functionally equivalent to `MemoryStore::default()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::store::MemoryStore;
    /// use libvctrl_handler::{Hash, ObjectStore};
    ///
    /// let store = MemoryStore::new();
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// assert!(!store.exists(&hash).unwrap());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
        }
    }
}

impl ObjectStore for MemoryStore {
    /// Stores a raw object in memory under the given hash.
    ///
    /// # Design rationale
    /// If an object with the same hash already exists, it is overwritten. The
    /// data is copied into a new `Vec<u8>` to ensure the store owns its own
    /// independent buffer.
    ///
    /// # Errors
    /// This implementation is infallible, but returns a `Result` to satisfy the
    /// trait interface.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::store::MemoryStore;
    /// use libvctrl_handler::{Hash, ObjectStore};
    ///
    /// let mut store = MemoryStore::new();
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// store.put(&hash, b"blob").unwrap();
    /// ```
    fn put(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError> {
        let _ = self.objects.insert(*hash, data.to_vec());
        Ok(())
    }

    /// Retrieves a raw object from memory by its hash as a stream.
    ///
    /// # Design rationale
    /// Returns a `Box<dyn Read>`. The internal `Vec<u8>` is cloned and wrapped
    /// in a `std::io::Cursor`. This prevents the caller from holding a mutable
    /// reference to the store's internals and allows for streaming reads.
    ///
    /// # Errors
    /// Returns [`VctrlError::ObjectNotFound`](libvctrl_handler::VctrlError::ObjectNotFound)
    /// if no object exists for the given hash.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::store::MemoryStore;
    /// use libvctrl_handler::{Hash, ObjectStore};
    /// use std::io::Read;
    ///
    /// let mut store = MemoryStore::new();
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// store.put(&hash, b"blob").unwrap();
    ///
    /// let mut reader = store.get(&hash).unwrap();
    /// let mut buf = Vec::new();
    /// reader.read_to_end(&mut buf).unwrap();
    /// assert_eq!(buf, b"blob");
    /// ```
    fn get(&self, hash: &Hash) -> Result<Box<dyn Read + '_>, VctrlError> {
        self.objects
            .get(hash)
            .cloned()
            .map(|v| Box::new(Cursor::new(v)) as Box<dyn Read>)
            .ok_or(VctrlError::ObjectNotFound(*hash))
    }

    /// Deletes an object from memory by its hash.
    ///
    /// # Design rationale
    /// This operation is idempotent. If the hash does not exist, the method
    /// silently succeeds without returning an error.
    ///
    /// # Errors
    /// This implementation is infallible, but returns a `Result` to satisfy the
    /// trait interface.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::store::MemoryStore;
    /// use libvctrl_handler::{Hash, ObjectStore};
    ///
    /// let mut store = MemoryStore::new();
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// store.put(&hash, b"blob").unwrap();
    /// store.delete(&hash).unwrap();
    /// assert!(!store.exists(&hash).unwrap());
    /// ```
    fn delete(&mut self, hash: &Hash) -> Result<(), VctrlError> {
        let _ = self.objects.remove(hash);
        Ok(())
    }

    /// Checks if an object exists in memory.
    ///
    /// # Design rationale
    /// This is an O(1) operation that checks the keys of the `HashMap`.
    ///
    /// # Errors
    /// This implementation is infallible, but returns a `Result` to satisfy the
    /// trait interface.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::store::MemoryStore;
    /// use libvctrl_handler::{Hash, ObjectStore};
    ///
    /// let store = MemoryStore::new();
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// assert!(!store.exists(&hash).unwrap());
    /// ```
    fn exists(&self, hash: &Hash) -> Result<bool, VctrlError> {
        Ok(self.objects.contains_key(hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libvctrl_handler::HASH_LENGTH;
    use std::io::Read;

    fn dummy_hash(byte: u8) -> Hash {
        let mut arr = [byte; HASH_LENGTH];
        arr[0] = byte;
        Hash::from_bytes(&arr).unwrap()
    }

    #[test]
    fn put_and_get() {
        let mut store = MemoryStore::new();
        let hash = dummy_hash(1);
        let data = b"hello world";
        store.put(&hash, data).unwrap();
        assert!(store.exists(&hash).unwrap());
        let mut buf = Vec::new();
        store.get(&hash).unwrap().read_to_end(&mut buf).unwrap();
        assert_eq!(buf, data);
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
        assert!(store.delete(&hash).is_ok());
    }

    #[test]
    fn put_overwrites() {
        let mut store = MemoryStore::new();
        let hash = dummy_hash(5);
        store.put(&hash, b"old").unwrap();
        store.put(&hash, b"new").unwrap();
        let mut buf = Vec::new();
        store.get(&hash).unwrap().read_to_end(&mut buf).unwrap();
        assert_eq!(buf, b"new");
    }
}
