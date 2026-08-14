//! Named reference management (e.g., branches and tags).
//!
//! # Purpose
//!
//! This module defines the [`RefStore`] trait, which abstracts the storage
//! of named references in a version control system. A named reference is a
//! human-readable string (such as `"HEAD"`, `"refs/heads/main"`, or
//! `"refs/tags/v1.0.0"`) that maps to a specific `Hash` value. References
//! are the mechanism by which mutable pointers to immutable objects are
//! maintained.
//!
//! # Design Rationale
//!
//! References are stored separately from the object database
//! ([`ObjectStore`]) because they have fundamentally
//! different lifecycle and mutability characteristics:
//!
//! - **Objects are immutable**: Once an object is written, its content hash
//!   cannot change without changing its identity. Objects are never updated
//!   in place.
//! - **References are mutable**: A branch or tag may be updated to point to
//!   a different hash over time. Deleting a reference is also a common
//!   operation.
//!
//! Keeping references separate from objects allows efficient updates without
//! touching the object store. It also enables different backends for
//! references (e.g., loose ref files, packed-refs, or a database table)
//! while the object store may be a completely different system.
//!
//! # Associated Type: `RefsIterator`
//!
//! The [`RefStore`] trait defines an associated type `RefsIterator` that
//! must implement [`Iterator<Item = Result<String, VctrlError>>`]. This
//! design allows implementations to choose the most appropriate iterator
//! type for their storage backend:
//!
//! - A simple in-memory implementation may use
//!   [`std::vec::IntoIter`].
//! - A disk-backed implementation may stream directory entries lazily.
//! - A database-backed implementation may return a custom iterator over
//!   rows.
//!
//! The associated type is bound to yield fallible results because listing
//! references may encounter I/O errors at any point during iteration.
//!
//! # Internal Mechanism
//!
//! The trait methods mirror typical key-value operations:
//!
//! - [`set_ref`](RefStore::set_ref) inserts or overwrites a name-to-hash
//!   mapping.
//! - [`get_ref`](RefStore::get_ref) looks up the hash for a name, returning
//!   [`VctrlError::RefNotFound`] if absent.
//! - [`delete_ref`](RefStore::delete_ref) removes a mapping.
//! - [`list_refs`](RefStore::list_refs) enumerates all available names.
//!
//! # Error Handling
//!
//! All methods return [`Result<_, VctrlError>`] so that callers can handle
//! failures uniformly. The most important error variant for this trait is
//! [`VctrlError::RefNotFound`], which
//! signals that a requested reference does not exist.
//!
//! # Examples
//!
//! A complete in-memory reference store:
//!
//! ```
//! use libvctrl_handler::{Hash, RefStore, VctrlError};
//! use std::collections::HashMap;
//!
//! #[derive(Default)]
//! struct InMemoryRefs(HashMap<String, Hash>);
//!
//! impl RefStore for InMemoryRefs {
//!     type RefsIterator = std::vec::IntoIter<Result<String, VctrlError>>;
//!
//!     fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError> {
//!         self.0.insert(name.to_string(), *hash);
//!         Ok(())
//!     }
//!
//!     fn get_ref(&self, name: &str) -> Result<Hash, VctrlError> {
//!         self.0
//!             .get(name)
//!             .copied()
//!             .ok_or_else(|| VctrlError::RefNotFound(name.to_string()))
//!     }
//!
//!     fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError> {
//!         self.0.remove(name);
//!         Ok(())
//!     }
//!
//!     fn list_refs(&self) -> Result<Self::RefsIterator, VctrlError> {
//!         let mut names: Vec<_> = self.0.keys().cloned().collect();
//!         names.sort();
//!         Ok(names.into_iter().map(Ok).collect::<Vec<_>>().into_iter())
//!     }
//! }
//!
//! let mut refs = InMemoryRefs::default();
//! let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
//! refs.set_ref("main", &hash).unwrap();
//! assert_eq!(refs.get_ref("main").unwrap(), hash);
//! ```

use crate::errors::VctrlError;
use crate::types::hash::Hash;

/// Defines the interface for a named reference store.
///
/// # Purpose
///
/// A `RefStore` maps human-readable names (e.g., `"HEAD"`,
/// `"refs/heads/main"`) to specific `Hash` values. This allows tracking
/// branches and tags without scanning the entire object database. The trait
/// is the persistence contract for the mutable pointer layer of a version
/// control system.
///
/// # Design Rationale
///
/// References are stored separately from the [`ObjectStore`]
/// because they are mutable and frequently updated, whereas objects are
/// immutable and content-addressed. This separation avoids coupling the
/// write-heavy reference updates with the append-oriented object storage.
///
/// The associated type `RefsIterator` allows implementations to return any
/// iterator over reference names, enabling lazy or streaming listing where
/// appropriate. This prevents forcing all references into a contiguous
/// vector when the backing store could provide a more efficient enumeration.
///
/// # Why `&str` for Names?
///
/// Methods accept `&str` rather than an owned [`String`] for name parameters.
/// This design allows callers to pass string literals or borrowed substrings
/// without allocation. Implementations that need ownership can convert the
/// borrowed slice to a [`String`] internally.
///
/// # Why `&Hash` for Values?
///
/// Methods accept `Hash` references to avoid copying a 64-byte value on
/// every call. The borrowed hash can be dereferenced and copied only when
/// the implementation actually needs to store it, minimizing stack traffic.
///
/// # Error Handling
///
/// Each method returns [`Result<_, VctrlError>`] to maintain a unified error
/// surface. The most common error conditions are:
///
/// - [`VctrlError::RefNotFound`] when a
///   requested name does not exist.
/// - [`VctrlError::IoError`] when the underlying
///   storage fails.
///
/// # Internal Mechanism
///
/// A typical implementation stores mappings in a [`HashMap`] or similar
/// structure. [`set_ref`](Self::set_ref) inserts or updates a mapping.
/// [`get_ref`](Self::get_ref) performs a lookup and returns the hash if
/// present. [`delete_ref`](Self::delete_ref) removes the mapping.
/// [`list_refs`](Self::list_refs) returns an iterator over all names,
/// usually in a deterministic order for testability.
///
/// # Examples
///
/// A complete in-memory implementation:
///
/// ```
/// use libvctrl_handler::{Hash, RefStore, VctrlError};
/// use std::collections::HashMap;
///
/// #[derive(Default)]
/// struct InMemoryRefs(HashMap<String, Hash>);
///
/// impl RefStore for InMemoryRefs {
///     type RefsIterator = std::vec::IntoIter<Result<String, VctrlError>>;
///
///     fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError> {
///         self.0.insert(name.to_string(), *hash);
///         Ok(())
///     }
///
///     fn get_ref(&self, name: &str) -> Result<Hash, VctrlError> {
///         self.0
///             .get(name)
///             .copied()
///             .ok_or_else(|| VctrlError::RefNotFound(name.to_string()))
///     }
///
///     fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError> {
///         self.0.remove(name);
///         Ok(())
///     }
///
///     fn list_refs(&self) -> Result<Self::RefsIterator, VctrlError> {
///         let mut names: Vec<_> = self.0.keys().cloned().collect();
///         names.sort();
///         Ok(names.into_iter().map(Ok).collect::<Vec<_>>().into_iter())
///     }
/// }
///
/// let mut refs = InMemoryRefs::default();
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// refs.set_ref("main", &hash).unwrap();
/// assert_eq!(refs.get_ref("main").unwrap(), hash);
/// ```
pub trait RefStore {
    /// An iterator over all reference names, yielding
    /// `Result<String, VctrlError>`.
    ///
    /// # Purpose
    ///
    /// This associated type defines the iterator type returned by
    /// [`list_refs`](RefStore::list_refs). It is constrained to yield
    /// fallible results so that implementations can propagate I/O errors
    /// encountered while enumerating references.
    ///
    /// # Why an Associated Type?
    ///
    /// Different storage backends have different natural iterator types. A
    /// memory-backed store may use [`std::vec::IntoIter`], while a database
    /// store may stream rows from a cursor. By exposing this as an
    /// associated type, the trait remains generic over the specific
    /// iterator while keeping the method signatures unified.
    ///
    /// # Requirements
    ///
    /// The iterator must implement
    /// [`Iterator<Item = Result<String, VctrlError>>`](Iterator). Each item
    /// is a reference name as a [`String`], or an error if enumeration
    /// fails partway.
    ///
    /// # Examples
    ///
    /// A valid associated type:
    ///
    /// ```
    /// use libvctrl_handler::{Hash, RefStore, VctrlError};
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct MyRefs(HashMap<String, Hash>);
    /// # impl RefStore for MyRefs {
    /// type RefsIterator = std::vec::IntoIter<Result<String, VctrlError>>;
    /// #     fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError> {
    /// #         self.0.insert(name.to_string(), *hash);
    /// #         Ok(())
    /// #     }
    /// #     fn get_ref(&self, name: &str) -> Result<Hash, VctrlError> {
    /// #         self.0.get(name).copied().ok_or_else(|| VctrlError::RefNotFound(name.to_string()))
    /// #     }
    /// #     fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError> {
    /// #         self.0.remove(name);
    /// #         Ok(())
    /// #     }
    /// #     fn list_refs(&self) -> Result<Self::RefsIterator, VctrlError> {
    /// #         Ok(vec![].into_iter())
    /// #     }
    /// # }
    /// ```
    type RefsIterator: Iterator<Item = Result<String, VctrlError>>;

    /// Sets or updates a named reference to point to a specific hash.
    ///
    /// # Purpose
    ///
    /// Inserts a new reference name-to-hash mapping, or overwrites the
    /// existing mapping if the name already exists. This is the primary
    /// write operation for branches, tags, and `HEAD`.
    ///
    /// # Arguments
    ///
    /// * `name` - The reference name (e.g., `"main"`). It is borrowed as
    ///   `&str` to avoid unnecessary allocation.
    /// * `hash` - The target hash value. It is borrowed as `&Hash` to avoid
    ///   copying the 64-byte value unless needed.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::IoError`] if the underlying storage fails to
    /// write the reference, such as when the disk is full or the file is
    /// read-only.
    ///
    /// # How It Works Internally
    ///
    /// The implementation typically converts the borrowed `name` into an
    /// owned [`String`] and inserts it into a map with a copy of the hash.
    /// If the name already exists, the previous value is replaced. The
    /// method returns `Ok(())` when the mapping is successfully stored.
    ///
    /// # Examples
    ///
    /// Setting a reference:
    ///
    /// ```
    /// # use libvctrl_handler::{Hash, RefStore, VctrlError};
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct Refs(HashMap<String, Hash>);
    /// # impl RefStore for Refs {
    /// #     type RefsIterator = std::vec::IntoIter<Result<String, VctrlError>>;
    /// #     fn set_ref(&mut self, n: &str, h: &Hash) -> Result<(), VctrlError> {
    /// #         self.0.insert(n.to_string(), *h); Ok(())
    /// #     }
    /// #     fn get_ref(&self, n: &str) -> Result<Hash, VctrlError> {
    /// #         self.0.get(n).copied().ok_or_else(|| VctrlError::RefNotFound(n.to_string()))
    /// #     }
    /// #     fn delete_ref(&mut self, n: &str) -> Result<(), VctrlError> {
    /// #         self.0.remove(n); Ok(())
    /// #     }
    /// #     fn list_refs(&self) -> Result<Self::RefsIterator, VctrlError> {
    /// #         Ok(vec![].into_iter())
    /// #     }
    /// # }
    /// let mut refs = Refs::default();
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// refs.set_ref("HEAD", &hash).unwrap();
    /// assert_eq!(refs.get_ref("HEAD").unwrap(), hash);
    /// ```
    fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError>;

    /// Retrieves the hash a named reference points to.
    ///
    /// # Purpose
    ///
    /// Looks up a reference by name and returns the associated hash. This is
    /// the primary read operation for resolving branch and tag names to
    /// object hashes.
    ///
    /// # Arguments
    ///
    /// * `name` - The reference name to resolve, borrowed as `&str`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(hash)` if the reference exists, or
    /// [`VctrlError::RefNotFound`] if it does not.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::RefNotFound`] if the reference does not exist.
    /// Returns [`VctrlError::IoError`] if the underlying storage fails to
    /// read, for example because the file is unreadable.
    ///
    /// # How It Works Internally
    ///
    /// The implementation looks up the name in its internal storage. For an
    /// in-memory store, this is a simple map lookup. The hash is copied
    /// (cheaply, because `Hash` is `Copy`) and returned.
    ///
    /// # Examples
    ///
    /// Retrieving a reference:
    ///
    /// ```
    /// # use libvctrl_handler::{Hash, RefStore, VctrlError};
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct Refs(HashMap<String, Hash>);
    /// # impl RefStore for Refs {
    /// #     type RefsIterator = std::vec::IntoIter<Result<String, VctrlError>>;
    /// #     fn set_ref(&mut self, n: &str, h: &Hash) -> Result<(), VctrlError> {
    /// #         self.0.insert(n.to_string(), *h); Ok(())
    /// #     }
    /// #     fn get_ref(&self, n: &str) -> Result<Hash, VctrlError> {
    /// #         self.0.get(n).copied().ok_or_else(|| VctrlError::RefNotFound(n.to_string()))
    /// #     }
    /// #     fn delete_ref(&mut self, n: &str) -> Result<(), VctrlError> {
    /// #         self.0.remove(n); Ok(())
    /// #     }
    /// #     fn list_refs(&self) -> Result<Self::RefsIterator, VctrlError> {
    /// #         Ok(vec![].into_iter())
    /// #     }
    /// # }
    /// let mut refs = Refs::default();
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// refs.set_ref("HEAD", &hash).unwrap();
    /// assert_eq!(refs.get_ref("HEAD").unwrap(), hash);
    /// ```
    fn get_ref(&self, name: &str) -> Result<Hash, VctrlError>;

    /// Deletes a named reference.
    ///
    /// # Purpose
    ///
    /// Removes a name-to-hash mapping from the store. After a successful
    /// delete, subsequent calls to [`get_ref`](Self::get_ref) with the same
    /// name must return [`VctrlError::RefNotFound`].
    ///
    /// # Arguments
    ///
    /// * `name` - The reference name to remove, borrowed as `&str`.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::IoError`] if the underlying storage fails to
    /// delete the reference, such as when the file is locked or the disk is
    /// read-only.
    ///
    /// # How It Works Internally
    ///
    /// The implementation removes the mapping from its internal data
    /// structure. For an in-memory store, this is a map removal. For a
    /// disk-backed store, it may unlink one or more files. Deleting a
    /// non-existent reference should succeed silently; it is not an error.
    ///
    /// # Examples
    ///
    /// Deleting a reference:
    ///
    /// ```
    /// # use libvctrl_handler::{Hash, RefStore, VctrlError};
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct Refs(HashMap<String, Hash>);
    /// # impl RefStore for Refs {
    /// #     type RefsIterator = std::vec::IntoIter<Result<String, VctrlError>>;
    /// #     fn set_ref(&mut self, n: &str, h: &Hash) -> Result<(), VctrlError> {
    /// #         self.0.insert(n.to_string(), *h); Ok(())
    /// #     }
    /// #     fn get_ref(&self, n: &str) -> Result<Hash, VctrlError> {
    /// #         self.0.get(n).copied().ok_or_else(|| VctrlError::RefNotFound(n.to_string()))
    /// #     }
    /// #     fn delete_ref(&mut self, n: &str) -> Result<(), VctrlError> {
    /// #         self.0.remove(n); Ok(())
    /// #     }
    /// #     fn list_refs(&self) -> Result<Self::RefsIterator, VctrlError> {
    /// #         Ok(vec![].into_iter())
    /// #     }
    /// # }
    /// let mut refs = Refs::default();
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// refs.set_ref("HEAD", &hash).unwrap();
    /// refs.delete_ref("HEAD").unwrap();
    /// assert!(refs.get_ref("HEAD").is_err());
    /// ```
    fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError>;

    /// Lists all reference names currently stored.
    ///
    /// # Purpose
    ///
    /// Returns an iterator over every reference name known to the store.
    /// This is useful for enumerating branches, tags, and other symbolic
    /// names, for example when building a repository browser or performing
    /// garbage collection.
    ///
    /// # Returns
    ///
    /// Returns `Ok(iterator)` where `iterator` yields
    /// [`Result<String, VctrlError>`] items, or an error if the initial
    /// listing fails.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::IoError`] if the underlying storage fails to
    /// read the list of references.
    ///
    /// # How It Works Internally
    ///
    /// The implementation collects all available names and returns them as
    /// an iterator. For deterministic behavior, it is recommended to sort
    /// the names before returning them, especially for in-memory stores,
    /// but this is not a strict requirement. A lazy implementation may
    /// instead yield names as they are discovered from the backing store.
    ///
    /// # Examples
    ///
    /// Listing references:
    ///
    /// ```
    /// # use libvctrl_handler::{Hash, RefStore, VctrlError};
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct Refs(HashMap<String, Hash>);
    /// # impl RefStore for Refs {
    /// #     type RefsIterator = std::vec::IntoIter<Result<String, VctrlError>>;
    /// #     fn set_ref(&mut self, n: &str, h: &Hash) -> Result<(), VctrlError> {
    /// #         self.0.insert(n.to_string(), *h); Ok(())
    /// #     }
    /// #     fn get_ref(&self, n: &str) -> Result<Hash, VctrlError> {
    /// #         self.0.get(n).copied().ok_or_else(|| VctrlError::RefNotFound(n.to_string()))
    /// #     }
    /// #     fn delete_ref(&mut self, n: &str) -> Result<(), VctrlError> {
    /// #         self.0.remove(n); Ok(())
    /// #     }
    /// #     fn list_refs(&self) -> Result<Self::RefsIterator, VctrlError> {
    /// #         let mut names: Vec<_> = self.0.keys().cloned().collect();
    /// #         names.sort();
    /// #         Ok(names.into_iter().map(Ok).collect::<Vec<_>>().into_iter())
    /// #     }
    /// # }
    /// let mut refs = Refs::default();
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// refs.set_ref("main", &hash).unwrap();
    /// refs.set_ref("dev", &hash).unwrap();
    ///
    /// let iter = refs.list_refs().unwrap();
    /// let mut names: Vec<_> = iter.collect::<Result<Vec<_>, _>>().unwrap();
    /// names.sort();
    /// assert_eq!(names, vec!["dev".to_string(), "main".to_string()]);
    /// ```
    fn list_refs(&self) -> Result<Self::RefsIterator, VctrlError>;
}
