//! Reference store trait.
//!
//! # Architecture
//! This module defines the abstract contract for managing Git references (branches,
//! tags, HEAD). In Git's architecture, the object database is strictly immutable,
//! while references provide the mutable pointers that track the current state of
//! branches and tags. By isolating reference management into a dedicated trait,
//! the crate decouples state mutations from content storage.
//!
//! # Design Rationale: Lazy Iteration
//! The [`RefStore::list_refs`] method returns a custom associated iterator type
//! (`type RefsIterator`) rather than a `Vec<String>`. This is a critical architectural
//! decision for scalability. Repositories like the Linux kernel contain millions of
//! references. Returning a `Vec` would require loading all names into memory
//! simultaneously, risking out-of-memory (OOM) errors. By returning an iterator,
//! backends can stream reference names lazily from disk or a database cursor,
//! maintaining a constant memory footprint.

use crate::errors::VctrlError;
use crate::types::Hash;

/// A trait for managing Git references (branches, tags, etc.).
///
/// # Why this exists
/// Provides a unified, type-safe interface for mutating and querying repository
/// state. Git references map human-readable names (e.g., `refs/heads/main`) to
/// cryptographic hashes. This trait enforces that structure, allowing the core
/// engine to orchestrate branch updates, tag creation, and HEAD detachments
/// without being tied to a specific filesystem layout or database backend.
///
/// # How it works
/// The store maintains a mapping between reference names and [`Hash`] values.
/// Write operations (`set_ref`, `delete_ref`) require `&mut self`, enforcing
/// exclusive access at the Rust type level. This mimics Git's `.lock` files,
/// preventing race conditions where two concurrent processes try to update the
/// same branch. Read operations (`get_ref`, `list_refs`) take `&self`, allowing
/// highly concurrent parallel reads across multiple threads.
///
/// # Design Rationale: Thread Safety
/// The trait requires `Send + Sync`. Reference resolution is one of the most
/// frequent operations in Git (e.g., during revision walks or merge analysis).
/// By enforcing thread safety, the engine can parallelize operations that
/// require resolving multiple refs without requiring external locking mechanisms.
///
/// # Examples
///
/// Implementing the trait for a mock in-memory store:
///
/// ```
/// # use libvctrl_handler::traits::core::ref_store::RefStore;
/// # use libvctrl_handler::{Hash, VctrlError};
/// # use std::collections::HashMap;
/// #
/// #[derive(Default)]
/// struct MockRefStore {
///     refs: HashMap<String, Hash>,
/// }
///
/// impl RefStore for MockRefStore {
///     type RefsIterator = std::vec::IntoIter<Result<String, VctrlError>>;
///
///     fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError> {
///         self.refs.insert(name.to_string(), *hash);
///         Ok(())
///     }
///
///     fn get_ref(&self, name: &str) -> Result<Hash, VctrlError> {
///         self.refs
///             .get(name)
///             .copied()
///             .ok_or_else(|| VctrlError::RefNotFound(name.to_string()))
///     }
///
///     fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError> {
///         self.refs.remove(name);
///         Ok(())
///     }
///
///     fn list_refs(&self) -> Result<Self::RefsIterator, VctrlError> {
///         let refs: Vec<_> = self.refs.keys().map(|k| Ok(k.clone())).collect();
///         Ok(refs.into_iter())
///     }
/// }
///
/// let mut store = MockRefStore::default();
/// let hash = Hash::from_bytes(&[0_u8; 64])?;
/// store.set_ref("refs/heads/main", &hash)?;
/// assert_eq!(store.get_ref("refs/heads/main")?, hash);
/// # Ok::<(), VctrlError>(())
/// ```
pub trait RefStore: Send + Sync {
    /// An iterator over reference names.
    ///
    /// # Why this exists
    /// Allows the backend to define its own iteration mechanism. A filesystem backend
    /// might yield names lazily via directory traversal, while a database backend
    /// might use a cursor. The iterator yields `Result<String, VctrlError>` to gracefully
    /// handle I/O errors that may occur mid-iteration (e.g., a permissions error on a
    /// specific file). The `Send` bound allows the iterator to be moved across threads.
    type RefsIterator: Iterator<Item = Result<String, VctrlError>> + Send;

    /// Sets a reference to the given hash.
    ///
    /// # How it works
    /// Inserts or updates the mapping of `name` to `hash`. If a reference with the
    /// given name already exists, it is overwritten. Requires `&mut self` to enforce
    /// exclusive access, preventing data races during concurrent branch updates.
    /// Implementors should ensure this operation is atomic to prevent repository
    /// corruption if the process is interrupted.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the underlying storage fails to persist the update
    /// (e.g., disk full, permission denied) or if the name is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::ref_store::RefStore;
    /// # use libvctrl_handler::{Hash, VctrlError};
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct MockRefStore { refs: HashMap<String, Hash> }
    /// # impl RefStore for MockRefStore {
    /// #     type RefsIterator = std::vec::IntoIter<Result<String, VctrlError>>;
    /// #     fn set_ref(&mut self, n: &str, h: &Hash) -> Result<(), VctrlError> { self.refs.insert(n.to_string(), *h); Ok(()) }
    /// #     fn get_ref(&self, n: &str) -> Result<Hash, VctrlError> { self.refs.get(n).copied().ok_or_else(|| VctrlError::RefNotFound(n.to_string())) }
    /// #     fn delete_ref(&mut self, n: &str) -> Result<(), VctrlError> { self.refs.remove(n); Ok(()) }
    /// #     fn list_refs(&self) -> Result<Self::RefsIterator, VctrlError> { let v: Vec<_> = self.refs.keys().map(|k| Ok(k.clone())).collect(); Ok(v.into_iter()) }
    /// # }
    /// let mut store = MockRefStore::default();
    /// let hash = Hash::from_bytes(&[1u8; 64])?;
    /// store.set_ref("refs/heads/feature", &hash)?;
    /// assert!(store.get_ref("refs/heads/feature").is_ok());
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError>;

    /// Gets the hash pointed to by a reference.
    ///
    /// # How it works
    /// Looks up the reference by name and returns the corresponding [`Hash`]. Takes
    /// `&self` to allow concurrent reads. If the reference does not exist, it returns
    /// an error rather than an `Option`, as a missing reference is typically an
    /// exceptional condition in Git operations (e.g., trying to checkout a non-existent
    /// branch).
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::RefNotFound`] if the reference name does not exist in the store.
    /// Returns [`VctrlError`] if the underlying storage cannot be read.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::ref_store::RefStore;
    /// # use libvctrl_handler::{Hash, VctrlError};
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct MockRefStore { refs: HashMap<String, Hash> }
    /// # impl RefStore for MockRefStore {
    /// #     type RefsIterator = std::vec::IntoIter<Result<String, VctrlError>>;
    /// #     fn set_ref(&mut self, n: &str, h: &Hash) -> Result<(), VctrlError> { self.refs.insert(n.to_string(), *h); Ok(()) }
    /// #     fn get_ref(&self, n: &str) -> Result<Hash, VctrlError> { self.refs.get(n).copied().ok_or_else(|| VctrlError::RefNotFound(n.to_string())) }
    /// #     fn delete_ref(&mut self, n: &str) -> Result<(), VctrlError> { self.refs.remove(n); Ok(()) }
    /// #     fn list_refs(&self) -> Result<Self::RefsIterator, VctrlError> { let v: Vec<_> = self.refs.keys().map(|k| Ok(k.clone())).collect(); Ok(v.into_iter()) }
    /// # }
    /// let mut store = MockRefStore::default();
    /// let hash = Hash::from_bytes(&[2u8; 64])?;
    /// store.set_ref("HEAD", &hash)?;
    /// assert_eq!(store.get_ref("HEAD")?, hash);
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn get_ref(&self, name: &str) -> Result<Hash, VctrlError>;

    /// Deletes a reference.
    ///
    /// # How it works
    /// Removes the mapping for the given `name`. If the reference does not exist,
    /// this operation is typically idempotent and returns `Ok(())`, preventing
    /// spurious errors during cleanup operations. Requires `&mut self` to enforce
    /// exclusive access.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the underlying storage cannot be modified.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::ref_store::RefStore;
    /// # use libvctrl_handler::{Hash, VctrlError};
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct MockRefStore { refs: HashMap<String, Hash> }
    /// # impl RefStore for MockRefStore {
    /// #     type RefsIterator = std::vec::IntoIter<Result<String, VctrlError>>;
    /// #     fn set_ref(&mut self, n: &str, h: &Hash) -> Result<(), VctrlError> { self.refs.insert(n.to_string(), *h); Ok(()) }
    /// #     fn get_ref(&self, n: &str) -> Result<Hash, VctrlError> { self.refs.get(n).copied().ok_or_else(|| VctrlError::RefNotFound(n.to_string())) }
    /// #     fn delete_ref(&mut self, n: &str) -> Result<(), VctrlError> { self.refs.remove(n); Ok(()) }
    /// #     fn list_refs(&self) -> Result<Self::RefsIterator, VctrlError> { let v: Vec<_> = self.refs.keys().map(|k| Ok(k.clone())).collect(); Ok(v.into_iter()) }
    /// # }
    /// let mut store = MockRefStore::default();
    /// let hash = Hash::from_bytes(&[3u8; 64])?;
    /// store.set_ref("refs/tags/v1", &hash)?;
    /// store.delete_ref("refs/tags/v1")?;
    /// assert!(store.get_ref("refs/tags/v1").is_err());
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError>;

    /// Lists all reference names.
    ///
    /// # How it works
    /// Returns a custom iterator ([`RefsIterator`](Self::RefsIterator)) that yields
    /// reference names. The iterator allows the backend to lazily load references,
    /// preventing memory exhaustion in repositories with a massive number of refs.
    /// Takes `&self` to allow concurrent listing.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the iterator cannot be initialized (e.g., an I/O
    /// error while opening the refs directory). Note that I/O errors occurring
    /// *during* iteration are yielded by the iterator itself as `Err(VctrlError)`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::ref_store::RefStore;
    /// # use libvctrl_handler::{Hash, VctrlError};
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct MockRefStore { refs: HashMap<String, Hash> }
    /// # impl RefStore for MockRefStore {
    /// #     type RefsIterator = std::vec::IntoIter<Result<String, VctrlError>>;
    /// #     fn set_ref(&mut self, n: &str, h: &Hash) -> Result<(), VctrlError> { self.refs.insert(n.to_string(), *h); Ok(()) }
    /// #     fn get_ref(&self, n: &str) -> Result<Hash, VctrlError> { self.refs.get(n).copied().ok_or_else(|| VctrlError::RefNotFound(n.to_string())) }
    /// #     fn delete_ref(&mut self, n: &str) -> Result<(), VctrlError> { self.refs.remove(n); Ok(()) }
    /// #     fn list_refs(&self) -> Result<Self::RefsIterator, VctrlError> { let v: Vec<_> = self.refs.keys().map(|k| Ok(k.clone())).collect(); Ok(v.into_iter()) }
    /// # }
    /// let mut store = MockRefStore::default();
    /// let hash = Hash::from_bytes(&[4u8; 64])?;
    /// store.set_ref("refs/heads/main", &hash)?;
    /// store.set_ref("refs/heads/dev", &hash)?;
    ///
    /// let refs: Vec<String> = store.list_refs()?.filter_map(|r| r.ok()).collect();
    /// assert_eq!(refs.len(), 2);
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn list_refs(&self) -> Result<Self::RefsIterator, VctrlError>;
}
