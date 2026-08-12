//! Content-addressable object storage with streaming reads.

use crate::errors::VctrlError;
use crate::types::hash::Hash;
use std::io::Read;

/// Defines the interface for a content-addressable object database.
///
/// # Purpose
///
/// An `ObjectStore` is responsible for storing and retrieving raw, serialized
/// version control objects (blobs, trees, commits, tags) using their
/// [`Hash`] as the primary key.
///
/// # Design Rationale
///
/// The trait uses `&Hash` for lookups rather than owned `Hash` values to
/// avoid unnecessary stack copies (64 bytes per hash). The `put` method
/// accepts a `&[u8]` slice, keeping the store agnostic to the serialization
/// format. The `get` method returns a [`Box<dyn Read>`] instead of a
/// concrete byte vector, enabling streaming and preventing large contiguous
/// allocations.
///
/// # Streaming semantics (`get`)
///
/// Implementations of `get` should return a reader that yields the exact
/// byte content of the stored object. The reader is borrowed from `&self`,
/// so the store cannot be mutated (e.g., via `put` or `delete`) while a
/// reader exists—this is enforced by Rust’s borrow checker. Callers must
/// consume the reader (e.g., via [`Read::read_to_end`]) to obtain the raw
/// bytes.
///
/// # Examples
///
/// A complete in-memory implementation:
///
/// ```
/// use libvctrl_handler::{Hash, ObjectStore, VctrlError};
/// use std::collections::HashMap;
/// use std::io::Read;
///
/// #[derive(Default)]
/// struct InMemoryStore(HashMap<Hash, Vec<u8>>);
///
/// impl ObjectStore for InMemoryStore {
///     fn put(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError> {
///         self.0.insert(*hash, data.to_vec());
///         Ok(())
///     }
///
///     fn get(&self, hash: &Hash) -> Result<Box<dyn Read + '_>, VctrlError> {
///         self.0
///             .get(hash)
///             .cloned()
///             .map(|v| Box::new(std::io::Cursor::new(v)) as Box<dyn Read>)
///             .ok_or_else(|| VctrlError::ObjectNotFound(*hash))
///     }
///
///     fn delete(&mut self, hash: &Hash) -> Result<(), VctrlError> {
///         self.0.remove(hash);
///         Ok(())
///     }
///
///     fn exists(&self, hash: &Hash) -> Result<bool, VctrlError> {
///         Ok(self.0.contains_key(hash))
///     }
/// }
///
/// let mut store = InMemoryStore::default();
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// store.put(&hash, b"data").unwrap();
///
/// // Read back the object using the streaming interface
/// let mut reader = store.get(&hash).unwrap();
/// let mut buf = Vec::new();
/// reader.read_to_end(&mut buf).unwrap();
/// assert_eq!(buf, b"data");
/// ```
pub trait ObjectStore {
    /// Stores a raw object in the database under the given hash.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::IoError`] if the underlying storage fails to write.
    /// Returns [`VctrlError::Other`] for implementation-specific failures.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Hash, ObjectStore, VctrlError};
    /// # use std::collections::HashMap;
    /// # use std::io::Read;
    /// # #[derive(Default)]
    /// # struct Store(HashMap<Hash, Vec<u8>>);
    /// # impl ObjectStore for Store {
    /// #     fn put(&mut self, h: &Hash, d: &[u8]) -> Result<(), VctrlError> {
    /// #         self.0.insert(*h, d.to_vec()); Ok(())
    /// #     }
    /// #     fn get(&self, h: &Hash) -> Result<Box<dyn Read + '_>, VctrlError> {
    /// #         self.0.get(h).cloned().map(|v| Box::new(std::io::Cursor::new(v)) as Box<dyn Read>).ok_or_else(|| VctrlError::ObjectNotFound(*h))
    /// #     }
    /// #     fn delete(&mut self, h: &Hash) -> Result<(), VctrlError> { self.0.remove(h); Ok(()) }
    /// #     fn exists(&self, h: &Hash) -> Result<bool, VctrlError> { Ok(self.0.contains_key(h)) }
    /// # }
    /// let mut s = Store::default();
    /// let h = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// s.put(&h, b"blob").unwrap();
    /// ```
    fn put(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError>;

    /// Retrieves a raw object from the database by its hash.
    ///
    /// The returned reader provides streaming access to the object bytes.
    /// Use [`Read::read_to_end`] or other [`Read`] methods to consume the
    /// data incrementally.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::ObjectNotFound`] if no object exists for the hash.
    /// Returns [`VctrlError::IoError`] if the underlying storage fails to read.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Hash, ObjectStore, VctrlError};
    /// # use std::collections::HashMap;
    /// # use std::io::Read;
    /// # #[derive(Default)]
    /// # struct Store(HashMap<Hash, Vec<u8>>);
    /// # impl ObjectStore for Store {
    /// #     fn put(&mut self, h: &Hash, d: &[u8]) -> Result<(), VctrlError> {
    /// #         self.0.insert(*h, d.to_vec()); Ok(())
    /// #     }
    /// #     fn get(&self, h: &Hash) -> Result<Box<dyn Read + '_>, VctrlError> {
    /// #         self.0.get(h).cloned().map(|v| Box::new(std::io::Cursor::new(v)) as Box<dyn Read>).ok_or_else(|| VctrlError::ObjectNotFound(*h))
    /// #     }
    /// #     fn delete(&mut self, h: &Hash) -> Result<(), VctrlError> { self.0.remove(h); Ok(()) }
    /// #     fn exists(&self, h: &Hash) -> Result<bool, VctrlError> { Ok(self.0.contains_key(h)) }
    /// # }
    /// let mut s = Store::default();
    /// let h = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// s.put(&h, b"blob").unwrap();
    ///
    /// let mut reader = s.get(&h).unwrap();
    /// let mut data = Vec::new();
    /// reader.read_to_end(&mut data).unwrap();
    /// assert_eq!(data, b"blob");
    /// ```
    fn get(&self, hash: &Hash) -> Result<Box<dyn Read + '_>, VctrlError>;

    /// Deletes an object from the database by its hash.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::IoError`] if the underlying storage fails to delete.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Hash, ObjectStore, VctrlError};
    /// # use std::collections::HashMap;
    /// # use std::io::Read;
    /// # #[derive(Default)]
    /// # struct Store(HashMap<Hash, Vec<u8>>);
    /// # impl ObjectStore for Store {
    /// #     fn put(&mut self, h: &Hash, d: &[u8]) -> Result<(), VctrlError> {
    /// #         self.0.insert(*h, d.to_vec()); Ok(())
    /// #     }
    /// #     fn get(&self, h: &Hash) -> Result<Box<dyn Read + '_>, VctrlError> {
    /// #         self.0.get(h).cloned().map(|v| Box::new(std::io::Cursor::new(v)) as Box<dyn Read>).ok_or_else(|| VctrlError::ObjectNotFound(*h))
    /// #     }
    /// #     fn delete(&mut self, h: &Hash) -> Result<(), VctrlError> { self.0.remove(h); Ok(()) }
    /// #     fn exists(&self, h: &Hash) -> Result<bool, VctrlError> { Ok(self.0.contains_key(h)) }
    /// # }
    /// let mut s = Store::default();
    /// let h = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// s.put(&h, b"blob").unwrap();
    /// s.delete(&h).unwrap();
    /// assert!(!s.exists(&h).unwrap());
    /// ```
    fn delete(&mut self, hash: &Hash) -> Result<(), VctrlError>;

    /// Checks if an object exists in the database.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::IoError`] if the underlying storage fails to check.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Hash, ObjectStore, VctrlError};
    /// # use std::collections::HashMap;
    /// # use std::io::Read;
    /// # #[derive(Default)]
    /// # struct Store(HashMap<Hash, Vec<u8>>);
    /// # impl ObjectStore for Store {
    /// #     fn put(&mut self, h: &Hash, d: &[u8]) -> Result<(), VctrlError> {
    /// #         self.0.insert(*h, d.to_vec()); Ok(())
    /// #     }
    /// #     fn get(&self, h: &Hash) -> Result<Box<dyn Read + '_>, VctrlError> {
    /// #         self.0.get(h).cloned().map(|v| Box::new(std::io::Cursor::new(v)) as Box<dyn Read>).ok_or_else(|| VctrlError::ObjectNotFound(*h))
    /// #     }
    /// #     fn delete(&mut self, h: &Hash) -> Result<(), VctrlError> { self.0.remove(h); Ok(()) }
    /// #     fn exists(&self, h: &Hash) -> Result<bool, VctrlError> { Ok(self.0.contains_key(h)) }
    /// # }
    /// let s = Store::default();
    /// let h = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// assert!(!s.exists(&h).unwrap());
    /// ```
    fn exists(&self, hash: &Hash) -> Result<bool, VctrlError>;
}
