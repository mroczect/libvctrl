//! Object storage trait.
//!
//! # Architecture
//! This module defines the abstract contract for a Content-Addressable Storage (CAS)
//! backend. In a CAS system, the identifier of an object is derived directly from its
//! content (typically via a cryptographic hash). This trait abstracts the underlying
//! storage mechanism, allowing the engine to use loose files on disk, packed objects,
//! or entirely in-memory representations.
//!
//! # Design Rationale: Streaming I/O
//! The `get` method returns a `Box<dyn Read>` rather than a `Vec<u8>` or `&[u8]`.
//! This is a critical architectural decision for performance and memory safety. Git
//! objects, particularly blobs, can be gigabytes in size. Loading an entire object
//! into memory could cause severe memory fragmentation and potential out-of-memory
//! (OOM) errors. By returning a reader, the storage backend allows the caller to
//! stream the data in fixed-size chunks, maintaining a constant memory footprint
//! regardless of the object's size.

use crate::errors::VctrlError;
use crate::types::Hash;
use std::io::Read;

/// A trait for storing and retrieving Git objects.
///
/// # Why this exists
/// Provides the fundamental contract for interacting with the Git object database.
/// By using a trait, the crate decouples the core VCS logic from the specific I/O
/// backend. This allows consumers to inject custom backends (e.g., S3 storage,
/// encrypted databases, or mock memory stores for testing) without altering the
/// core algorithms.
///
/// # How it works
/// The store maps [`Hash`] keys to raw byte payloads. Write operations (`put`,
/// `delete`) require `&mut self`, enforcing exclusive access to prevent data races
/// during mutations. Read operations (`get`, `exists`) take `&self`, allowing
/// highly concurrent parallel reads across multiple threads.
///
/// # Design Rationale: Thread Safety
/// The trait requires `Send + Sync`. Object storage is frequently accessed by
/// multiple concurrent operations (e.g., packing objects, resolving diffs, checking
/// out files). The `Send + Sync` bound guarantees that the implementor is thread-safe,
/// enabling the engine to parallelize object retrieval without external synchronization.
///
/// # Examples
///
/// Implementing the trait for a mock in-memory store:
///
/// ```
/// # use libvctrl_handler::traits::core::object_store::ObjectStore;
/// # use libvctrl_handler::{Hash, VctrlError};
/// # use std::collections::HashMap;
/// # use std::io::Cursor;
/// #
/// #[derive(Default)]
/// struct MockStore {
///     data: HashMap<Hash, Vec<u8>>,
/// }
///
/// impl ObjectStore for MockStore {
///     fn put(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError> {
///         self.data.insert(*hash, data.to_vec());
///         Ok(())
///     }
///
///     fn get(&self, hash: &Hash) -> Result<Box<dyn Read + Send + '_>, VctrlError> {
///         match self.data.get(hash) {
///             Some(data) => Ok(Box::new(Cursor::new(data.clone()))),
///             None => Err(VctrlError::ObjectNotFound(*hash)),
///         }
///     }
///
///     fn delete(&mut self, hash: &Hash) -> Result<(), VctrlError> {
///         self.data.remove(hash);
///         Ok(())
///     }
///
///     fn exists(&self, hash: &Hash) -> Result<bool, VctrlError> {
///         Ok(self.data.contains_key(hash))
///     }
/// }
///
/// let mut store = MockStore::default();
/// let hash = Hash::from_bytes(&[0_u8; 64])?;
/// store.put(&hash, b"blob content")?;
/// assert!(store.exists(&hash)?);
/// # Ok::<(), VctrlError>(())
/// ```
pub trait ObjectStore: Send + Sync {
    /// Stores an object under the given hash.
    ///
    /// # How it works
    /// Accepts a reference to the [`Hash`] and a byte slice of the object's raw,
    /// uncompressed content. The implementor is responsible for persisting this
    /// data (e.g., writing to disk, compressing into a packfile, or inserting
    /// into a database). Requires `&mut self` as it mutates the underlying storage.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the underlying storage fails (e.g., disk full,
    /// permission denied) or if the data violates storage constraints.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::object_store::ObjectStore;
    /// # use libvctrl_handler::{Hash, VctrlError};
    /// # use std::collections::HashMap;
    /// # use std::io::Cursor;
    /// # #[derive(Default)]
    /// # struct MockStore { data: HashMap<Hash, Vec<u8>> }
    /// # impl ObjectStore for MockStore {
    /// #     fn put(&mut self, h: &Hash, d: &[u8]) -> Result<(), VctrlError> { self.data.insert(*h, d.to_vec()); Ok(()) }
    /// #     fn get(&self, h: &Hash) -> Result<Box<dyn Read + Send + '_>, VctrlError> { match self.data.get(h) { Some(d) => Ok(Box::new(Cursor::new(d.clone()))), None => Err(VctrlError::ObjectNotFound(*h)) } }
    /// #     fn delete(&mut self, h: &Hash) -> Result<(), VctrlError> { self.data.remove(h); Ok(()) }
    /// #     fn exists(&self, h: &Hash) -> Result<bool, VctrlError> { Ok(self.data.contains_key(h)) }
    /// # }
    /// let mut store = MockStore::default();
    /// let hash = Hash::from_bytes(&[1u8; 64])?;
    /// store.put(&hash, b"new data")?;
    /// assert!(store.exists(&hash)?);
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn put(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError>;

    /// Retrieves an object by hash, returning a reader.
    ///
    /// # How it works
    /// Looks up the object by its [`Hash`] and returns a boxed reader. The reader
    /// abstracts the underlying storage medium (file handle, network socket, or
    /// memory cursor). The lifetime `'_` ties the returned reader to the lifetime
    /// of the `ObjectStore` instance, ensuring the underlying storage remains valid
    /// while the stream is active. This prevents loading large objects into memory
    /// all at once.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::ObjectNotFound`] if the hash does not exist in the store.
    /// Returns [`VctrlError`] if an I/O error occurs while initializing the stream.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::object_store::ObjectStore;
    /// # use libvctrl_handler::{Hash, VctrlError};
    /// # use std::collections::HashMap;
    /// # use std::io::{Cursor, Read};
    /// # #[derive(Default)]
    /// # struct MockStore { data: HashMap<Hash, Vec<u8>> }
    /// # impl ObjectStore for MockStore {
    /// #     fn put(&mut self, h: &Hash, d: &[u8]) -> Result<(), VctrlError> { self.data.insert(*h, d.to_vec()); Ok(()) }
    /// #     fn get(&self, h: &Hash) -> Result<Box<dyn Read + Send + '_>, VctrlError> { match self.data.get(h) { Some(d) => Ok(Box::new(Cursor::new(d.clone()))), None => Err(VctrlError::ObjectNotFound(*h)) } }
    /// #     fn delete(&mut self, h: &Hash) -> Result<(), VctrlError> { self.data.remove(h); Ok(()) }
    /// #     fn exists(&self, h: &Hash) -> Result<bool, VctrlError> { Ok(self.data.contains_key(h)) }
    /// # }
    /// let mut store = MockStore::default();
    /// let hash = Hash::from_bytes(&[2u8; 64])?;
    /// store.put(&hash, b"readable data")?;
    ///
    /// let mut reader = store.get(&hash)?;
    /// let mut content = String::new();
    /// reader.read_to_string(&mut content)?;
    /// assert_eq!(content, "readable data");
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn get(&self, hash: &Hash) -> Result<Box<dyn Read + Send + '_>, VctrlError>;

    /// Deletes an object by hash.
    ///
    /// # How it works
    /// Locates the object by its [`Hash`] and removes it from the underlying storage.
    /// If the object does not exist, this operation is typically idempotent and
    /// returns `Ok(())`, preventing spurious errors during garbage collection.
    /// Requires `&mut self` to enforce exclusive access during mutation.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the underlying storage cannot be modified (e.g.,
    /// file permission issues or read-only filesystem).
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::object_store::ObjectStore;
    /// # use libvctrl_handler::{Hash, VctrlError};
    /// # use std::collections::HashMap;
    /// # use std::io::Cursor;
    /// # #[derive(Default)]
    /// # struct MockStore { data: HashMap<Hash, Vec<u8>> }
    /// # impl ObjectStore for MockStore {
    /// #     fn put(&mut self, h: &Hash, d: &[u8]) -> Result<(), VctrlError> { self.data.insert(*h, d.to_vec()); Ok(()) }
    /// #     fn get(&self, h: &Hash) -> Result<Box<dyn Read + Send + '_>, VctrlError> { match self.data.get(h) { Some(d) => Ok(Box::new(Cursor::new(d.clone()))), None => Err(VctrlError::ObjectNotFound(*h)) } }
    /// #     fn delete(&mut self, h: &Hash) -> Result<(), VctrlError> { self.data.remove(h); Ok(()) }
    /// #     fn exists(&self, h: &Hash) -> Result<bool, VctrlError> { Ok(self.data.contains_key(h)) }
    /// # }
    /// let mut store = MockStore::default();
    /// let hash = Hash::from_bytes(&[3u8; 64])?;
    /// store.put(&hash, b"to be deleted")?;
    /// store.delete(&hash)?;
    /// assert!(!store.exists(&hash)?);
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn delete(&mut self, hash: &Hash) -> Result<(), VctrlError>;

    /// Checks whether an object exists.
    ///
    /// # How it works
    /// Performs a lightweight existence check without retrieving the object's data
    /// or initializing a stream. This is significantly faster than calling `get`
    /// and checking for `ObjectNotFound`, especially on network-backed storage.
    /// Takes `&self` to allow concurrent existence checks.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the underlying storage cannot be queried (e.g.,
    /// an I/O error while listing directory contents).
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::object_store::ObjectStore;
    /// # use libvctrl_handler::{Hash, VctrlError};
    /// # use std::collections::HashMap;
    /// # use std::io::Cursor;
    /// # #[derive(Default)]
    /// # struct MockStore { data: HashMap<Hash, Vec<u8>> }
    /// # impl ObjectStore for MockStore {
    /// #     fn put(&mut self, h: &Hash, d: &[u8]) -> Result<(), VctrlError> { self.data.insert(*h, d.to_vec()); Ok(()) }
    /// #     fn get(&self, h: &Hash) -> Result<Box<dyn Read + Send + '_>, VctrlError> { match self.data.get(h) { Some(d) => Ok(Box::new(Cursor::new(d.clone()))), None => Err(VctrlError::ObjectNotFound(*h)) } }
    /// #     fn delete(&mut self, h: &Hash) -> Result<(), VctrlError> { self.data.remove(h); Ok(()) }
    /// #     fn exists(&self, h: &Hash) -> Result<bool, VctrlError> { Ok(self.data.contains_key(h)) }
    /// # }
    /// let store = MockStore::default();
    /// let hash = Hash::from_bytes(&[4u8; 64])?;
    /// // Check a missing object
    /// assert!(!store.exists(&hash)?);
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn exists(&self, hash: &Hash) -> Result<bool, VctrlError>;
}
