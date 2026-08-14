//! Content-addressable object storage with streaming reads.
//!
//! # Purpose
//!
//! This module defines the [`ObjectStore`] trait, which represents the
//! persistence layer for raw, serialized version control objects. An object
//! store is a key-value database where the key is a `Hash` (the content
//! address) and the value is the raw byte representation of a
//! [`Blob`], [`Tree`], [`Commit`],
//! or [`Tag`].
//!
//! # Design Rationale
//!
//! Separating object storage into a trait provides several benefits:
//!
//! - **Backend agnosticism**: Implementations can be in-memory, on-disk,
//!   remote, or backed by a database without altering the rest of the
//!   system.
//! - **Testability**: Dummy or in-memory stores simplify unit testing of
//!   higher-level components.
//! - **Streaming efficiency**: The [`ObjectStore::get`] method returns a
//!   reader instead of a [`Vec<u8>`], enabling large objects to be consumed
//!   incrementally without allocating their full contents at once.
//! - **Immutability focus**: Objects are content-addressed and therefore
//!   immutable once written. The trait does not expose update operations.
//!
//! # Streaming Semantics
//!
//! The [`ObjectStore::get`] method returns a [`Box<dyn std::io::Read>`].
//! This design allows callers to stream the object bytes directly from the
//! backing store. The reader is tied to the lifetime of `&self`, meaning the
//! store cannot be mutated while a reader is alive. This is enforced by
//! Rust's borrow checker and prevents data races in single-threaded code.
//!
//! # How It Works Internally
//!
//! An implementation stores byte vectors under `Hash` keys. When
//! [`ObjectStore::put`] is called, the implementation should copy or move
//! the provided `data` into its internal storage. When
//! [`ObjectStore::get`] is called, the implementation looks up the hash and
//! returns a reader over the stored bytes, or
//! [`VctrlError::ObjectNotFound`] if the hash does not exist. The
//! [`ObjectStore::delete`] and [`ObjectStore::exists`] methods provide
//! additional lifecycle management.
//!
//! # Examples
//!
//! A complete in-memory implementation demonstrates all methods:
//!
//! ```
//! use libvctrl_handler::{Hash, ObjectStore, VctrlError};
//! use std::collections::HashMap;
//! use std::io::Read;
//!
//! #[derive(Default)]
//! struct InMemoryStore(HashMap<Hash, Vec<u8>>);
//!
//! impl ObjectStore for InMemoryStore {
//!     fn put(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError> {
//!         self.0.insert(*hash, data.to_vec());
//!         Ok(())
//!     }
//!
//!     fn get(&self, hash: &Hash) -> Result<Box<dyn Read + '_>, VctrlError> {
//!         self.0
//!             .get(hash)
//!             .cloned()
//!             .map(|v| Box::new(std::io::Cursor::new(v)) as Box<dyn Read>)
//!             .ok_or_else(|| VctrlError::ObjectNotFound(*hash))
//!     }
//!
//!     fn delete(&mut self, hash: &Hash) -> Result<(), VctrlError> {
//!         self.0.remove(hash);
//!         Ok(())
//!     }
//!
//!     fn exists(&self, hash: &Hash) -> Result<bool, VctrlError> {
//!         Ok(self.0.contains_key(hash))
//!     }
//! }
//!
//! let mut store = InMemoryStore::default();
//! let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
//! store.put(&hash, b"data").unwrap();
//!
//! let mut reader = store.get(&hash).unwrap();
//! let mut buf = Vec::new();
//! reader.read_to_end(&mut buf).unwrap();
//! assert_eq!(buf, b"data");
//! ```

use crate::errors::VctrlError;
use crate::types::hash::Hash;
use std::io::Read;

/// Defines the interface for a content-addressable object database.
///
/// # Purpose
///
/// An `ObjectStore` is responsible for storing and retrieving raw,
/// serialized version control objects (blobs, trees, commits, tags) using
/// their `Hash` as the primary key. This trait is the low-level
/// persistence contract that all storage backends must implement.
///
/// # Design Rationale
///
/// - **`&Hash` lookups**: The trait uses borrowed `Hash` references for
///   lookups rather than owned values. A `Hash` is 64 bytes; borrowing
///   avoids unnecessary stack copies and permits the store to implement
///   efficient in-place key comparisons.
/// - **`&[u8]` for `put`**: The `put` method accepts a byte slice instead of
///   a [`Vec<u8>`] to avoid forcing ownership transfer. The implementation
///   may choose to copy, move, or stream the data into its internal storage.
/// - **Streaming reads**: The `get` method returns a
///   [`Box<dyn Read>`](std::io::Read) rather than a concrete byte vector.
///   This enables callers to process large objects incrementally and
///   prevents large contiguous allocations when only a portion of the data
///   is needed.
/// - **Immutable objects**: Objects are content-addressed, meaning their
///   hash is derived from their bytes. Mutating stored data would break the
///   hash invariant, so the trait does not provide an update method. The
///   store is conceptually append-only (with `delete` as the exception).
///
/// # Streaming Semantics (`get`)
///
/// Implementations of `get` should return a reader that yields the exact
/// byte content of the stored object. The reader is borrowed from `&self`,
/// so the store cannot be mutated (e.g., via `put` or `delete`) while a
/// reader exists. This is enforced by Rust's borrow checker. Callers must
/// consume the reader (e.g., via
/// [`Read::read_to_end`](std::io::Read::read_to_end)) to obtain the raw
/// bytes.
///
/// # Error Handling
///
/// All methods return a [`Result`] with [`VctrlError`]. This unifies error
/// handling across all backends and allows callers to match on specific
/// failure conditions such as
/// [`ObjectNotFound`](VctrlError::ObjectNotFound) or
/// [`IoError`](VctrlError::IoError).
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
    /// # Purpose
    ///
    /// This method writes the serialized bytes of an object into the store
    /// and associates them with the provided content hash. After this
    /// operation completes successfully, [`ObjectStore::get`] with the same
    /// hash must return the exact data that was stored.
    ///
    /// # Arguments
    ///
    /// * `hash` - The content address under which the object is stored.
    /// * `data` - The raw serialized bytes of the object.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::IoError`] if the underlying storage fails to
    /// write. Returns [`VctrlError::Other`] for implementation-specific
    /// failures such as quota exceeded or invalid data length.
    ///
    /// # How It Works Internally
    ///
    /// The implementation receives a borrowed slice and typically copies it
    /// into an internal buffer or writes it to disk. The hash is used as the
    /// key for later retrieval. The implementation should ensure that
    /// calling `put` with the same hash overwrites the previous content,
    /// because content-addressed systems may encounter duplicate writes
    /// (e.g., when the same object is encountered in multiple operations).
    ///
    /// # Examples
    ///
    /// Storing a blob in an in-memory store:
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
    /// let mut store = Store::default();
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// store.put(&hash, b"blob").unwrap();
    /// assert!(store.exists(&hash).unwrap());
    /// ```
    fn put(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError>;

    /// Retrieves a raw object from the database by its hash.
    ///
    /// The returned reader provides streaming access to the object bytes.
    /// Use [`Read::read_to_end`](std::io::Read::read_to_end) or other
    /// [`Read`](std::io::Read) methods to consume the data incrementally.
    ///
    /// # Purpose
    ///
    /// This method performs the primary read operation of an object store.
    /// It returns an object that implements [`Read`], allowing the caller
    /// to stream the stored bytes without forcing a full allocation.
    ///
    /// # Arguments
    ///
    /// * `hash` - The content address of the object to retrieve.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::ObjectNotFound`] if no object exists for the
    /// hash. Returns [`VctrlError::IoError`] if the underlying storage
    /// fails to read, for example due to permission errors or hardware
    /// failure.
    ///
    /// # How It Works Internally
    ///
    /// The implementation looks up the hash in its internal storage. If
    /// found, it constructs a reader over the stored bytes. In the example
    /// above, a [`std::io::Cursor`] is used to provide a reader over an
    /// in-memory byte vector. For on-disk stores, the reader may be a file
    /// handle or a decompression reader.
    ///
    /// # Borrow Checker Implications
    ///
    /// The returned reader borrows from `&self`, so the store cannot be
    /// mutated while the reader is alive. This ensures that the data being
    /// streamed remains consistent and is not deleted or overwritten
    /// concurrently.
    ///
    /// # Examples
    ///
    /// Reading an object back from the store:
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
    /// let mut store = Store::default();
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// store.put(&hash, b"blob").unwrap();
    ///
    /// let mut reader = store.get(&hash).unwrap();
    /// let mut data = Vec::new();
    /// reader.read_to_end(&mut data).unwrap();
    /// assert_eq!(data, b"blob");
    /// ```
    fn get(&self, hash: &Hash) -> Result<Box<dyn Read + '_>, VctrlError>;

    /// Deletes an object from the database by its hash.
    ///
    /// # Purpose
    ///
    /// Removes the object associated with the given hash from the store.
    /// After this operation, [`ObjectStore::get`] with the same hash must
    /// return [`VctrlError::ObjectNotFound`]. Deletion is useful for
    /// garbage collection and repository maintenance, although in a purely
    /// content-addressed system, objects are often kept indefinitely.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::IoError`] if the underlying storage fails to
    /// delete, such as when the file is locked or the disk is read-only.
    ///
    /// # How It Works Internally
    ///
    /// The implementation removes the entry corresponding to the hash from
    /// its internal data structure. For in-memory stores this is a simple
    /// map removal; for disk-backed stores it may involve unlinking one or
    /// more files.
    ///
    /// # Examples
    ///
    /// Deleting an object from the store:
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
    /// let mut store = Store::default();
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// store.put(&hash, b"blob").unwrap();
    /// store.delete(&hash).unwrap();
    /// assert!(!store.exists(&hash).unwrap());
    /// ```
    fn delete(&mut self, hash: &Hash) -> Result<(), VctrlError>;

    /// Checks if an object exists in the database.
    ///
    /// # Purpose
    ///
    /// Returns whether an object with the given hash is present in the
    /// store. This method allows callers to avoid an expensive read or to
    /// implement conditional logic based on object availability.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::IoError`] if the underlying storage fails to
    /// perform the existence check, for example due to directory access
    /// errors.
    ///
    /// # How It Works Internally
    ///
    /// The implementation performs a key lookup in its internal storage and
    /// returns `true` if the key exists, `false` otherwise. It does not
    /// require reading the full object data.
    ///
    /// # Examples
    ///
    /// Checking for object existence:
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
    /// let store = Store::default();
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// assert!(!store.exists(&hash).unwrap());
    /// ```
    fn exists(&self, hash: &Hash) -> Result<bool, VctrlError>;
}
