//! In-memory object storage backend for `libvctrl_core`.
//!
//! # Purpose
//!
//! This module provides the [`MemoryStore`], a concrete implementation of the
//! [`ObjectStore`](libvctrl_handler::ObjectStore) trait. It uses a standard
//! [`HashMap`] to store serialized version control objects in RAM.
//!
//! # Design Rationale
//!
//! - **Ephemeral storage**: As a non-persistent store, it is ideal for unit
//!   testing, temporary caching, or short-lived sessions where disk or
//!   network I/O is unnecessary or undesirable.
//! - **Streaming reads**: The `get` method returns a `Box<dyn Read>`, which
//!   wraps the data in a [`std::io::Cursor`]. This design allows the caller
//!   to stream the object's contents incrementally, which is crucial for
//!   large blobs that should not be loaded entirely into memory at once.
//! - **Idempotent deletion**: The `delete` method acts as a no-op if the
//!   object does not exist, preventing errors during cleanup and simplifying
//!   caller logic.
//!
//! # Internal Mechanism
//!
//! The store maps a 64-byte [`Hash`](libvctrl_handler::Hash) to a `Vec<u8>`.
//! Data is copied on insertion (`put`) into a new owned buffer. On retrieval
//! (`get`), the internal `Vec<u8>` is cloned and wrapped in a
//! [`std::io::Cursor`]. This cloning ensures that the caller receives an
//! independent snapshot of the data and does not hold a lock on the store's
//! internal structures.
//!
//! # Why Clone on Read?
//!
//! The `get` method clones the stored byte vector before wrapping it in a
//! cursor. This is a deliberate trade-off:
//!
//! - **Borrow checker simplicity**: The returned reader owns its data, so it
//!   does not borrow from `self`. This allows the store to be mutated (e.g.,
//!   via `put` or `delete`) even while a reader is still alive, avoiding
//!   complex lifetime annotations.
//! - **Snapshot semantics**: The caller receives an immutable snapshot of the
//!   object. Subsequent modifications to the store do not affect the reader's
//!   data, which is often the desired behavior in concurrent or iterative
//!   scenarios.
//! - **Cost**: For large blobs, cloning can be expensive. However, for an
//!   in-memory testing and caching backend, the simplicity and safety
//!   outweigh the performance cost. Persistent backends can implement
//!   zero-copy streaming differently.
//!
//! # Complexity
//!
//! - `put`: average O(1) insertion into a [`HashMap`].
//! - `get`: average O(1) lookup plus O(n) clone of the data.
//! - `delete`: average O(1) removal.
//! - `exists`: average O(1) key check.
//!
//! # Examples
//!
//! Storing and retrieving an object via streaming:
//!
//! ```
//! use libvctrl_core::store::MemoryStore;
//! use libvctrl_handler::{Hash, ObjectStore};
//! use std::io::Read;
//!
//! let mut store = MemoryStore::new();
//! let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
//!
//! store.put(&hash, b"my data").unwrap();
//! assert!(store.exists(&hash).unwrap());
//!
//! let mut reader = store.get(&hash).unwrap();
//! let mut buf = Vec::new();
//! reader.read_to_end(&mut buf).unwrap();
//! assert_eq!(buf, b"my data");
//! ```

use libvctrl_handler::{Hash, ObjectStore, VctrlError};
use std::collections::HashMap;
use std::io::{Cursor, Read};

/// An in-memory implementation of the
/// [`ObjectStore`](libvctrl_handler::ObjectStore) trait.
///
/// # Purpose
///
/// Stores version control objects in a [`HashMap`] residing in RAM. This
/// backend is primarily intended for testing and ephemeral operations where
/// persistence is not required.
///
/// # Design Rationale
///
/// This struct derives [`Default`] to allow easy instantiation via
/// `MemoryStore::default()` or `MemoryStore::new()`. The internal [`HashMap`]
/// is kept private to ensure that all modifications go through the
/// `ObjectStore` trait implementation, preserving the integrity of the
/// storage interface.
///
/// # Field Privacy
///
/// The `objects` field is private. External code cannot directly access or
/// mutate the internal map; all operations must go through the trait methods.
/// This encapsulation prevents accidental bypass of the storage contract.
///
/// # Memory Layout
///
/// The store owns a [`HashMap`] where keys are [`Hash`] values (64-byte
/// arrays, `Copy`) and values are [`Vec<u8>`] buffers. The map is allocated
/// on the heap, and its capacity grows dynamically as objects are inserted.
///
/// # Thread Safety
///
/// `MemoryStore` is not [`Sync`] because [`HashMap`] itself is not safe for
/// concurrent access. If shared access is needed, wrap it in a [`Mutex`] or
/// [`RwLock`].
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
    /// # Design Rationale
    ///
    /// This is a standard constructor that initializes an empty [`HashMap`].
    /// It is functionally equivalent to `MemoryStore::default()`. The
    /// constructor takes no arguments and performs no allocation until the
    /// first object is inserted.
    ///
    /// # Performance
    ///
    /// Creating the store is O(1) and does not allocate heap memory for the
    /// map because [`HashMap::new`] defers allocation until the first
    /// insertion.
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
    /// # Purpose
    ///
    /// Inserts the provided byte slice into the store, keyed by the given
    /// [`Hash`]. If an object with the same hash already exists, its data is
    /// overwritten.
    ///
    /// # Design Rationale
    ///
    /// The data is copied into a new [`Vec<u8>`] via `data.to_vec()`. This
    /// ensures that the store owns its own independent buffer. The caller
    /// retains ownership of the original slice and may modify or drop it
    /// without affecting the stored object.
    ///
    /// # Complexity
    ///
    /// Average O(1) insertion plus O(n) copy of the input bytes.
    ///
    /// # Errors
    ///
    /// This implementation is infallible, but returns a `Result` to satisfy
    /// the [`ObjectStore`](libvctrl_handler::ObjectStore) trait interface.
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
    /// # Purpose
    ///
    /// Returns a boxed reader over the stored bytes for the given hash. The
    /// reader implements [`Read`](std::io::Read), allowing the caller to
    /// consume the data incrementally.
    ///
    /// # Design Rationale
    ///
    /// The internal [`Vec<u8>`] is cloned and wrapped in a
    /// [`std::io::Cursor`]. This provides an independent snapshot of the
    /// data, so the reader does not borrow from `self`. This avoids lifetime
    /// entanglement and permits subsequent mutation of the store while the
    /// reader is still alive.
    ///
    /// # Complexity
    ///
    /// Average O(1) lookup plus O(n) clone of the data, where n is the
    /// object size.
    ///
    /// # Errors
    ///
    /// Returns
    /// [`VctrlError::ObjectNotFound`](libvctrl_handler::VctrlError::ObjectNotFound)
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
    /// # Purpose
    ///
    /// Removes the object associated with the given hash from the store.
    /// After this operation, subsequent calls to `get` with the same hash
    /// return `ObjectNotFound`.
    ///
    /// # Design Rationale
    ///
    /// This operation is idempotent. If the hash does not exist, the method
    /// silently succeeds without returning an error. This simplifies cleanup
    /// logic by allowing callers to attempt deletion unconditionally.
    ///
    /// # Complexity
    ///
    /// Average O(1) removal.
    ///
    /// # Errors
    ///
    /// This implementation is infallible, but returns a `Result` to satisfy
    /// the [`ObjectStore`](libvctrl_handler::ObjectStore) trait interface.
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
    /// # Purpose
    ///
    /// Returns whether the store contains an object for the given hash
    /// without retrieving or cloning the data. This is useful for conditional
    /// logic and validation before performing expensive operations.
    ///
    /// # Design Rationale
    ///
    /// This is an O(1) key check on the underlying [`HashMap`]. The method
    /// does not perform any heap allocation or data copying, making it
    /// extremely cheap.
    ///
    /// # Complexity
    ///
    /// Average O(1).
    ///
    /// # Errors
    ///
    /// This implementation is infallible, but returns a `Result` to satisfy
    /// the [`ObjectStore`](libvctrl_handler::ObjectStore) trait interface.
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
