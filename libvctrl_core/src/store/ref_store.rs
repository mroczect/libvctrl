//! In‑memory reference store – the simplest [`RefStore`] implementation.
//!
//! [`MemoryRefStore`] implements [`RefStore`] using a `HashMap<String, Hash>`.
//! It validates reference names before storing them, as required by the
//! trait contract.
//!
//! # Design
//!
//! References are mutable named pointers to objects. They are used to
//! track branches, tags, and special refs like `HEAD`. This store
//! implements the full [`RefStore`] trait with a simple `HashMap`.
//!
//! # Name validation
//!
//! Names are validated on every call to [`set_ref`](MemoryRefStore::set_ref):
//! - Must not be empty.
//! - Must not exceed [`MAX_NAME_LENGTH`](libvctrl_handler::MAX_NAME_LENGTH).
//! - No additional character restrictions are applied (this is a
//!   reference implementation; production code may add path traversal
//!   checks or other policies).
//!
//! # Performance
//!
//! - **`set_ref`**: O(1) average.
//! - **`get_ref`**: O(1) average.
//! - **`delete_ref`**: O(1) average.
//! - **`list_ref`s**: O(n) where n is the number of references.
//!
//! # Examples
//!
//! ```rust
//! use libvctrl_core::store::MemoryRefStore;
//! use libvctrl_handler::{Hash, RefStore, HASH_LENGTH};
//!
//! let mut refs = MemoryRefStore::new();
//! let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
//!
//! // Set a reference
//! refs.set_ref("refs/heads/main", &hash).unwrap();
//!
//! // Look up a reference
//! assert_eq!(refs.get_ref("refs/heads/main").unwrap(), hash);
//!
//! // List all references
//! let all = refs.list_refs().unwrap();
//! assert!(all.contains(&"refs/heads/main".to_string()));
//!
//! // Delete a reference
//! refs.delete_ref("refs/heads/main").unwrap();
//! assert!(refs.get_ref("refs/heads/main").is_err());
//! ```

use libvctrl_handler::{Hash, MAX_NAME_LENGTH, RefStore, VctrlError};
use std::collections::HashMap;

/// An in‑memory reference store.
///
/// Validates names before storing them, as required by the trait contract.
///
/// # Characteristics
/// - **Not thread‑safe**: wrap in `Arc<Mutex<…>>` for shared access.
/// - **Not persistent**: all data is lost when the store is dropped.
///
/// # Examples
/// ```
/// use libvctrl_core::store::MemoryRefStore;
/// use libvctrl_handler::{Hash, RefStore, HASH_LENGTH};
///
/// let mut refs = MemoryRefStore::new();
/// let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
/// refs.set_ref("HEAD", &hash).unwrap();
/// assert_eq!(refs.get_ref("HEAD").unwrap(), hash);
/// ```
#[derive(Debug, Default)]
pub struct MemoryRefStore {
    refs: HashMap<String, Hash>,
}

impl MemoryRefStore {
    /// Creates an empty reference store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            refs: HashMap::new(),
        }
    }
}

impl RefStore for MemoryRefStore {
    /// Create or update a reference.
    ///
    /// # Errors
    /// Returns [`VctrlError::InvalidName`] if `name` is empty or exceeds
    /// [`MAX_NAME_LENGTH`](libvctrl_handler::MAX_NAME_LENGTH).
    fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError> {
        if name.is_empty() || name.len() > MAX_NAME_LENGTH {
            return Err(VctrlError::InvalidName(name.into()));
        }
        let _ = self.refs.insert(name.to_string(), *hash);
        Ok(())
    }

    /// Look up a reference by name.
    ///
    /// # Errors
    /// Returns [`VctrlError::RefNotFound`] if the reference does not exist.
    fn get_ref(&self, name: &str) -> Result<Hash, VctrlError> {
        self.refs
            .get(name)
            .copied()
            .ok_or_else(|| VctrlError::RefNotFound(name.into()))
    }

    /// Delete a reference.
    ///
    /// Deleting a non‑existent reference succeeds silently.
    ///
    /// # Errors
    /// This implementation never fails (it always returns `Ok(())`).
    fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError> {
        let _ = self.refs.remove(name);
        Ok(())
    }

    /// List all reference names currently stored.
    ///
    /// The order of the returned names is arbitrary.
    ///
    /// # Errors
    /// This implementation never fails (it always returns `Ok(Vec)`).
    fn list_refs(&self) -> Result<Vec<String>, VctrlError> {
        Ok(self.refs.keys().cloned().collect())
    }
}
