//! In-memory reference storage backend for `libvctrl_core`.
//!
//! # Purpose
//!
//! This module provides the [`MemoryRefStore`], a concrete implementation of the
//! [`RefStore`](libvctrl_handler::RefStore) trait. It uses a standard
//! [`HashMap`] to map human-readable reference names (e.g., "HEAD",
//! "refs/heads/main") to cryptographic [`Hash`](libvctrl_handler::Hash) values.
//!
//! # Design Rationale
//!
//! - **Ephemeral and mutable**: Unlike objects, which are immutable and
//!   content-addressed, references are frequently updated (e.g., when a new
//!   commit is made). This store provides a fast, ephemeral location for these
//!   mutations in RAM.
//! - **Separation from ObjectStore**: Keeping references in a separate store
//!   allows backends to optimize their storage layouts. For example, an object
//!   store might be content-addressable and read-only, while a ref store needs
//!   to support arbitrary name updates.
//! - **Input validation**: The `set_ref` method enforces name length constraints
//!   ([`MAX_NAME_LENGTH`](libvctrl_handler::MAX_NAME_LENGTH)) to prevent
//!   resource exhaustion and ensure compatibility with filesystem-based backends.
//! - **Deterministic iteration**: The `list_refs` method sorts the references
//!   before returning them. This ensures deterministic output, which is critical
//!   for reproducible testing and stable diffs.
//!
//! # Internal Mechanism
//!
//! The store maps a `String` to a 64-byte [`Hash`](libvctrl_handler::Hash).
//! Lookups, insertions, and deletions are average O(1) operations. The
//! [`Hash`](libvctrl_handler::Hash) is `Copy`, so retrieving a reference returns
//! a cheap stack copy rather than a heap allocation.
//!
//! # Complexity
//!
//! - `set_ref`: average O(1) insertion into a [`HashMap`].
//! - `get_ref`: average O(1) lookup.
//! - `delete_ref`: average O(1) removal.
//! - `list_refs`: O(n log n) due to sorting, where n is the number of
//!   references.
//!
//! # Examples
//!
//! Setting and retrieving a branch reference:
//!
//! ```
//! use libvctrl_core::store::MemoryRefStore;
//! use libvctrl_handler::{Hash, RefStore};
//!
//! let mut store = MemoryRefStore::new();
//! let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
//!
//! store.set_ref("refs/heads/main", &hash).unwrap();
//! assert_eq!(store.get_ref("refs/heads/main").unwrap(), hash);
//! ```

use libvctrl_handler::{Hash, RefStore, VctrlError};
use std::collections::HashMap;

/// An in-memory implementation of the
/// [`RefStore`](libvctrl_handler::RefStore) trait.
///
/// # Purpose
///
/// Stores version control references in a [`HashMap`] residing in RAM. This
/// backend is primarily intended for testing, ephemeral operations, and
/// in-memory state management.
///
/// # Design Rationale
///
/// This struct derives [`Default`] to allow easy instantiation. The internal
/// [`HashMap`] is kept private to ensure that all modifications go through the
/// `RefStore` trait implementation, preserving the integrity of the interface
/// and ensuring name validation is always enforced.
///
/// # Field Privacy
///
/// The `refs` field is private. External code cannot directly access or
/// mutate the internal map; all operations must go through the trait methods.
/// This encapsulation prevents accidental bypass of name validation and
/// preserves the invariants of the store.
///
/// # Memory Layout
///
/// The store owns a [`HashMap`] where keys are [`String`] names and values are
/// [`Hash`] values (64-byte arrays, `Copy`). The map is allocated on the heap,
/// and its capacity grows dynamically as references are inserted.
///
/// # Thread Safety
///
/// `MemoryRefStore` is not [`Sync`] because [`HashMap`] itself is not safe for
/// concurrent access. If shared access is needed, wrap it in a [`Mutex`] or
/// [`RwLock`].
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
    /// # Design Rationale
    ///
    /// This is a standard constructor that initializes an empty [`HashMap`].
    /// It is functionally equivalent to `MemoryRefStore::default()`. The
    /// constructor takes no arguments and performs no allocation until the
    /// first reference is inserted.
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
    type RefsIterator = std::vec::IntoIter<Result<String, VctrlError>>;

    /// Sets or updates a named reference to point to a specific hash.
    ///
    /// # Purpose
    ///
    /// Inserts or overwrites a mapping from a reference name to a target
    /// [`Hash`]. If the name already exists, its target is updated to the new
    /// hash. This operation supports both branch and tag updates.
    ///
    /// # Design Rationale
    ///
    /// This method enforces name validation by rejecting empty names or names
    /// exceeding [`MAX_NAME_LENGTH`](libvctrl_handler::MAX_NAME_LENGTH). If a
    /// reference with the same name already exists, its target hash is
    /// overwritten. The method takes `&str` for the name to allow borrowed
    /// string slices and converts it to an owned [`String`] for storage.
    ///
    /// # Complexity
    ///
    /// Average O(1) insertion plus O(k) copy of the name, where k is the
    /// name length.
    ///
    /// # Errors
    ///
    /// Returns
    /// [`VctrlError::InvalidName`](libvctrl_handler::VctrlError::InvalidName)
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
        if name.is_empty()
            || name.len()
                > usize::try_from(libvctrl_handler::MAX_NAME_LENGTH)
                    .expect("MAX_NAME_LENGTH too large")
        {
            return Err(VctrlError::InvalidName(name.into()));
        }
        let _ = self.refs.insert(name.to_string(), *hash);
        Ok(())
    }

    /// Retrieves the hash a named reference points to.
    ///
    /// # Purpose
    ///
    /// Looks up a reference by name and returns the associated [`Hash`]. This
    /// is the primary read operation for resolving branch or tag names to
    /// object hashes.
    ///
    /// # Design Rationale
    ///
    /// Returns a copied `Hash` value (which is `Copy`) rather than a reference,
    /// allowing the caller to use it without borrowing from the store. This
    /// avoids lifetime entanglement and permits subsequent mutation of the
    /// store while the returned hash is still in use.
    ///
    /// # Complexity
    ///
    /// Average O(1) lookup.
    ///
    /// # Errors
    ///
    /// Returns
    /// [`VctrlError::RefNotFound`](libvctrl_handler::VctrlError::RefNotFound)
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
    /// # Purpose
    ///
    /// Removes a reference from the store by name. After this operation,
    /// subsequent calls to `get_ref` with the same name return
    /// [`VctrlError::RefNotFound`].
    ///
    /// # Design Rationale
    ///
    /// This operation is idempotent. If the reference does not exist, the
    /// method silently succeeds without returning an error, simplifying
    /// cleanup logic. This mirrors the behavior of many map removal
    /// operations.
    ///
    /// # Complexity
    ///
    /// Average O(1) removal.
    ///
    /// # Errors
    ///
    /// This implementation is infallible, but returns a `Result` to satisfy
    /// the [`RefStore`](libvctrl_handler::RefStore) trait interface.
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
    /// # Purpose
    ///
    /// Returns an iterator over all reference names in the store. The names
    /// are sorted alphabetically to ensure deterministic output, which is
    /// important for tests and stable user-facing listings.
    ///
    /// # Design Rationale
    ///
    /// The method collects all keys from the internal [`HashMap`] into a
    /// [`Vec<String>`], sorts them alphabetically, and returns an iterator.
    /// The sorting step is crucial because [`HashMap`] iteration order is
    /// non-deterministic; without sorting, the order could vary between runs.
    ///
    /// The iterator yields `Result<String, VctrlError>` to satisfy the trait
    /// definition, which allows disk-based backends to yield I/O errors
    /// mid-iteration. This in-memory implementation always yields `Ok`.
    ///
    /// # Complexity
    ///
    /// O(n log n) due to sorting, plus O(n) for collecting and cloning names,
    /// where n is the number of references.
    ///
    /// # Errors
    ///
    /// This implementation is infallible, but returns a `Result` to satisfy
    /// the [`RefStore`](libvctrl_handler::RefStore) trait interface.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::store::MemoryRefStore;
    /// use libvctrl_handler::{Hash, RefStore};
    ///
    /// let mut store = MemoryRefStore::new();
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// store.set_ref("b", &hash).unwrap();
    /// store.set_ref("a", &hash).unwrap();
    ///
    /// let refs: Vec<String> = store.list_refs().unwrap().collect::<Result<Vec<_>, _>>().unwrap();
    /// assert_eq!(refs, vec!["a".to_string(), "b".to_string()]);
    /// ```
    fn list_refs(&self) -> Result<Self::RefsIterator, VctrlError> {
        let mut names: Vec<String> = self.refs.keys().cloned().collect();
        names.sort();
        Ok(names.into_iter().map(Ok).collect::<Vec<_>>().into_iter())
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
        let long_name = "a".repeat(
            usize::try_from(libvctrl_handler::MAX_NAME_LENGTH).expect("MAX_NAME_LENGTH too large")
                + 1,
        );
        assert!(store.set_ref(&long_name, &dummy_hash()).is_err());
    }

    #[test]
    fn list_refs() {
        let mut store = MemoryRefStore::new();
        store.set_ref("a", &dummy_hash()).unwrap();
        store.set_ref("b", &dummy_hash()).unwrap();
        let iter = store.list_refs().unwrap();
        let list: Vec<String> = iter.collect::<Result<Vec<_>, _>>().unwrap();
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
