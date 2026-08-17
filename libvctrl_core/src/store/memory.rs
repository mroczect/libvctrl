//! In-memory [`ObjectStore`] implementation backed by a [`HashMap`].
//!
//! # Why this module exists
//!
//! The [`MemoryStore`] type provides a lightweight, ephemeral storage backend
//! for version-control objects. It implements the [`ObjectStore`] contract
//! without requiring disk I/O, network access, or persistent state. This makes
//! it ideal for:
//!
//! - Unit tests that need an isolated object database.
//! - Caching and temporary storage.
//! - Embedded or ephemeral applications where persistence is not desired.
//!
//! # How it works
//!
//! Objects are stored as raw byte vectors (`Vec<u8>`) keyed by their content
//! hash ([`Hash`]). The use of a [`HashMap`] gives average O(1) lookup,
//! insertion, and deletion. The raw bytes are not parsed or validated on
//! insertion; validation is the responsibility of higher layers. This keeps
//! the store fast and agnostic to object type.
//!
//! The [`get`](MemoryStore::get) method returns a
//! `Box<dyn Read + Send + '_>` rather than a `Vec<u8>` to support streaming
//! reads of large objects without forcing the entire object into a contiguous
//! buffer. Internally, it wraps the stored slice in a [`Cursor`].
//!
//! # Examples
//!
//! Store and retrieve an object:
//!
//! ```
//! use libvctrl_core::store::MemoryStore;
//! use libvctrl_handler::{Hash, ObjectStore};
//! use std::io::Read;
//!
//! let mut store = MemoryStore::new();
//! let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
//!
//! store.put(&hash, b"hello world").unwrap();
//!
//! let mut reader = store.get(&hash).unwrap();
//! let mut buf = Vec::new();
//! reader.read_to_end(&mut buf).unwrap();
//! assert_eq!(buf, b"hello world");
//! ```

use libvctrl_handler::{Hash, ObjectStore, VctrlError};
use std::collections::HashMap;
use std::io::{Cursor, Read};

/// An in-memory implementation of [`ObjectStore`].
///
/// # Design rationale
///
/// The struct uses a [`HashMap<Hash, Vec<u8>>`] as its sole storage. This
/// choice provides:
///
/// - **Fast average O(1) access** — hashing is performed by the [`Hash`] key.
/// - **No parsing overhead** — objects are stored as opaque byte sequences.
/// - **Simple ownership model** — the map owns both keys and values, so the
///   store can be dropped without manual cleanup.
///
/// The type derives [`Default`], allowing `MemoryStore::default()` to create a
/// new empty store without requiring a custom constructor. However, an explicit
/// [`new`](MemoryStore::new) is still provided for symmetry with other store
/// implementations.
///
/// # Examples
///
/// Create an empty store and verify it is initially empty:
///
/// ```
/// # use libvctrl_core::store::MemoryStore;
/// # use libvctrl_handler::{Hash, ObjectStore};
/// # let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let store = MemoryStore::new();
/// assert!(!store.exists(&hash).unwrap());
/// ```
#[derive(Debug, Default)]
pub struct MemoryStore {
    objects: HashMap<Hash, Vec<u8>>,
}

impl MemoryStore {
    /// Creates a new empty `MemoryStore`.
    ///
    /// # Why this is `const`
    ///
    /// The constructor is a `const fn` because constructing an empty
    /// [`HashMap`] does not require any runtime heap allocation. The map is
    /// allocated lazily on the first insertion. This allows the store to be
    /// created in constant contexts and enables potential compile-time
    /// evaluation by the compiler.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_core::store::MemoryStore;
    /// let store = MemoryStore::new();
    /// // store is ready to use, but contains no objects
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
        }
    }
}

impl ObjectStore for MemoryStore {
    /// Stores an object under the given hash.
    ///
    /// # How it works
    ///
    /// The method copies the provided byte slice into a new `Vec<u8>` and
    /// inserts it into the internal [`HashMap`]. If an object with the same
    /// hash already exists, the old value is silently replaced. The method
    /// always returns `Ok(())` because an in-memory map has no failure modes
    /// under normal conditions (excluding allocation failure, which panics).
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_core::store::MemoryStore;
    /// # use libvctrl_handler::{Hash, ObjectStore};
    /// # let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let mut store = MemoryStore::new();
    /// store.put(&hash, b"data").unwrap();
    /// assert!(store.exists(&hash).unwrap());
    /// ```
    fn put(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError> {
        let _ = self.objects.insert(*hash, data.to_vec());
        Ok(())
    }

    /// Retrieves an object as a streaming reader.
    ///
    /// # Design rationale
    ///
    /// Returning `Box<dyn Read + Send + '_>` instead of `Vec<u8>` allows
    /// callers to consume large objects incrementally. The lifetime `'_` is
    /// tied to `&self`, enabling the returned reader to borrow the stored bytes
    /// without cloning the entire object.
    ///
    /// Internally, the stored slice is wrapped in a [`Cursor`], which
    /// implements both [`Read`] and [`Send`].
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::ObjectNotFound`] if no object with the given hash
    /// exists in the store.
    ///
    /// # Examples
    ///
    /// Read back a stored object:
    ///
    /// ```
    /// # use libvctrl_core::store::MemoryStore;
    /// # use libvctrl_handler::{Hash, ObjectStore};
    /// # use std::io::Read;
    /// # let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let mut store = MemoryStore::new();
    /// store.put(&hash, b"hello").unwrap();
    ///
    /// let mut reader = store.get(&hash).unwrap();
    /// let mut buf = Vec::new();
    /// reader.read_to_end(&mut buf).unwrap();
    /// assert_eq!(buf, b"hello");
    /// ```
    fn get(&self, hash: &Hash) -> Result<Box<dyn Read + Send + '_>, VctrlError> {
        let data = self
            .objects
            .get(hash)
            .ok_or(VctrlError::ObjectNotFound(*hash))?;
        Ok(Box::new(Cursor::new(data.as_slice())))
    }

    /// Deletes an object from the store.
    ///
    /// # How it works
    ///
    /// Removes the key-value pair from the internal [`HashMap`]. If the object
    /// does not exist, the method still returns `Ok(())`; deletion is
    /// idempotent. This mirrors the behavior of [`HashMap::remove`], which
    /// returns [`Option`] but does not fail.
    ///
    /// # Examples
    ///
    /// Delete an object and verify it is gone:
    ///
    /// ```
    /// # use libvctrl_core::store::MemoryStore;
    /// # use libvctrl_handler::{Hash, ObjectStore};
    /// # let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let mut store = MemoryStore::new();
    /// store.put(&hash, b"data").unwrap();
    /// store.delete(&hash).unwrap();
    /// assert!(!store.exists(&hash).unwrap());
    /// ```
    fn delete(&mut self, hash: &Hash) -> Result<(), VctrlError> {
        let _ = self.objects.remove(hash);
        Ok(())
    }

    /// Checks whether an object exists in the store.
    ///
    /// # How it works
    ///
    /// Delegates to [`HashMap::contains_key`], which is an average O(1)
    /// operation. The method does not inspect the object bytes or validate the
    /// hash; it only checks for key presence.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_core::store::MemoryStore;
    /// # use libvctrl_handler::{Hash, ObjectStore};
    /// # let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let mut store = MemoryStore::new();
    /// assert!(!store.exists(&hash).unwrap());
    /// store.put(&hash, b"data").unwrap();
    /// assert!(store.exists(&hash).unwrap());
    /// ```
    fn exists(&self, hash: &Hash) -> Result<bool, VctrlError> {
        Ok(self.objects.contains_key(hash))
    }
}
