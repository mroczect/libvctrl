//! Named reference management (e.g., branches and tags).

use crate::errors::VctrlError;
use crate::types::hash::Hash;

/// Defines the interface for a named reference store.
///
/// # Purpose
///
/// A `RefStore` maps human-readable names (e.g., "HEAD", "refs/heads/main")
/// to specific [`Hash`]es. This allows tracking branches and tags without
/// scanning the entire object database.
///
/// # Design Rationale
///
/// References are stored separately from the [`ObjectStore`] because they
/// are mutable and frequently updated, whereas objects are immutable and
/// content-addressed. The associated type `RefsIterator` allows implementations
/// to return any iterator over reference names, enabling lazy or streaming
/// listing where appropriate.
///
/// # Examples
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
///     fn get_ref(&self, name: &str) -> Result<Hash, VctrlError> {
///         self.0.get(name).copied().ok_or_else(|| VctrlError::RefNotFound(name.to_string()))
///     }
///     fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError> {
///         self.0.remove(name);
///         Ok(())
///     }
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
    /// An iterator over all reference names, yielding `Result<String, VctrlError>`.
    type RefsIterator: Iterator<Item = Result<String, VctrlError>>;

    /// Sets or updates a named reference to point to a specific hash.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::IoError`] if the underlying storage fails to write.
    ///
    /// # Examples
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
    /// #     fn delete_ref(&mut self, n: &str) -> Result<(), VctrlError> { self.0.remove(n); Ok(()) }
    /// #     fn list_refs(&self) -> Result<Self::RefsIterator, VctrlError> {
    /// #         Ok(vec![].into_iter())
    /// #     }
    /// # }
    /// let mut r = Refs::default();
    /// let h = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// r.set_ref("HEAD", &h).unwrap();
    /// ```
    fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError>;

    /// Retrieves the hash a named reference points to.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::RefNotFound`] if the reference does not exist.
    ///
    /// # Examples
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
    /// #     fn delete_ref(&mut self, n: &str) -> Result<(), VctrlError> { self.0.remove(n); Ok(()) }
    /// #     fn list_refs(&self) -> Result<Self::RefsIterator, VctrlError> {
    /// #         Ok(vec![].into_iter())
    /// #     }
    /// # }
    /// let mut r = Refs::default();
    /// let h = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// r.set_ref("HEAD", &h).unwrap();
    /// assert_eq!(r.get_ref("HEAD").unwrap(), h);
    /// ```
    fn get_ref(&self, name: &str) -> Result<Hash, VctrlError>;

    /// Deletes a named reference.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::IoError`] if the underlying storage fails to delete.
    ///
    /// # Examples
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
    /// #     fn delete_ref(&mut self, n: &str) -> Result<(), VctrlError> { self.0.remove(n); Ok(()) }
    /// #     fn list_refs(&self) -> Result<Self::RefsIterator, VctrlError> {
    /// #         Ok(vec![].into_iter())
    /// #     }
    /// # }
    /// let mut r = Refs::default();
    /// let h = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// r.set_ref("HEAD", &h).unwrap();
    /// r.delete_ref("HEAD").unwrap();
    /// assert!(r.get_ref("HEAD").is_err());
    /// ```
    fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError>;

    /// Lists all reference names currently stored.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::IoError`] if the underlying storage fails to read
    /// the list of references.
    ///
    /// # Examples
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
    /// #     fn delete_ref(&mut self, n: &str) -> Result<(), VctrlError> { self.0.remove(n); Ok(()) }
    /// #     fn list_refs(&self) -> Result<Self::RefsIterator, VctrlError> {
    /// #         let mut names: Vec<_> = self.0.keys().cloned().collect();
    /// #         names.sort();
    /// #         Ok(names.into_iter().map(Ok).collect::<Vec<_>>().into_iter())
    /// #     }
    /// # }
    /// let mut r = Refs::default();
    /// let h = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// r.set_ref("main", &h).unwrap();
    /// r.set_ref("dev", &h).unwrap();
    /// let iter = r.list_refs().unwrap();
    /// let mut names: Vec<_> = iter.collect::<Result<Vec<_>, _>>().unwrap();
    /// names.sort();
    /// assert_eq!(names, vec!["dev".to_string(), "main".to_string()]);
    /// ```
    fn list_refs(&self) -> Result<Self::RefsIterator, VctrlError>;
}
