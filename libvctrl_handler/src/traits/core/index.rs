//! Defines the `Index` trait for staging area operations.
//!
//! # Purpose
//!
//! The `Index` trait abstracts a version control staging area. It provides
//! methods to add, remove, and clear entries, write the current index into
//! a tree object, and read a tree object back into the index. This is the
//! core of commands like `git add`, `git write-tree`, and `git read-tree`.
//!
//! # Why a separate module
//!
//! The staging area is a distinct storage concern from object storage,
//! reference storage, and transport. Keeping the trait in its own file
//! allows different index backends (in-memory, file-based, etc.) to
//! implement the same contract without coupling the caller to a specific
//! implementation.
//!
//! # Examples
//!
//! A simple round-trip with a mock implementation:
//!
//! ```
//! use libvctrl_handler::{Hash, Index, VctrlError};
//!
//! #[derive(Clone)]
//! struct MockEntry {
//!     path: String,
//!     id: Hash,
//! }
//!
//! struct MockIndex {
//!     entries: Vec<MockEntry>,
//! }
//!
//! impl Index for MockIndex {
//!     type Entry = MockEntry;
//!     type Path = String;
//!     type TreeId = Hash;
//!
//!     fn add(&mut self, entry: MockEntry) -> Result<(), VctrlError> {
//!         self.entries.push(entry);
//!         Ok(())
//!     }
//!
//!     fn remove(&mut self, path: &String) -> Result<(), VctrlError> {
//!         self.entries.retain(|e| &e.path != path);
//!         Ok(())
//!     }
//!
//!     fn clear(&mut self) -> Result<(), VctrlError> {
//!         self.entries.clear();
//!         Ok(())
//!     }
//!
//!     fn write_tree(&self) -> Result<Hash, VctrlError> {
//!         Ok(self
//!             .entries
//!             .first()
//!             .map(|e| e.id)
//!             .unwrap_or_else(|| Hash::from_bytes(&[0u8; 64]).unwrap()))
//!     }
//!
//!     fn read_tree(&mut self, tree: &Hash) -> Result<(), VctrlError> {
//!         self.entries.clear();
//!         self.entries.push(MockEntry {
//!             path: "root".to_string(),
//!             id: *tree,
//!         });
//!         Ok(())
//!     }
//! }
//!
//! let hash = Hash::from_bytes(&[1u8; 64]).unwrap();
//! let mut index = MockIndex { entries: vec![] };
//!
//! index
//!     .add(MockEntry {
//!         path: "foo".to_string(),
//!         id: hash,
//!     })
//!     .unwrap();
//!
//! let tree = index.write_tree().unwrap();
//! assert_eq!(tree, hash);
//!
//! index.clear().unwrap();
//! assert!(index.entries.is_empty());
//!
//! index.read_tree(&tree).unwrap();
//! assert_eq!(index.entries.len(), 1);
//! ```

use crate::VctrlError;

/// Trait for managing a version control staging area.
///
/// # Purpose
///
/// The `Index` trait represents the staging area between the working
/// directory and the object database. It allows adding and removing entries,
/// clearing the index, writing the current index contents into a tree object,
/// and reading a tree object back into the index.
///
/// # Associated Types
///
/// - `Entry`: the type representing a single index entry.
/// - `Path`: the type used to address a path in the index.
/// - `TreeId`: the type used to identify the tree produced by `write_tree`.
///
/// # Examples
///
/// A trivial implementation that keeps an empty index:
///
/// ```
/// use libvctrl_handler::{Hash, Index, VctrlError};
///
/// struct EmptyIndex;
///
/// impl Index for EmptyIndex {
///     type Entry = ();
///     type Path = String;
///     type TreeId = Hash;
///
///     fn add(&mut self, _entry: ()) -> Result<(), VctrlError> {
///         Ok(())
///     }
///
///     fn remove(&mut self, _path: &String) -> Result<(), VctrlError> {
///         Ok(())
///     }
///
///     fn clear(&mut self) -> Result<(), VctrlError> {
///         Ok(())
///     }
///
///     fn write_tree(&self) -> Result<Hash, VctrlError> {
///         Ok(Hash::from_bytes(&[0u8; 64]).unwrap())
///     }
///
///     fn read_tree(&mut self, _tree: &Hash) -> Result<(), VctrlError> {
///         Ok(())
///     }
/// }
///
/// let mut index = EmptyIndex;
/// index.add(()).unwrap();
/// index.clear().unwrap();
/// let _ = index.write_tree().unwrap();
/// ```
///
/// # Errors
///
/// - [`VctrlError::Other`] if the underlying index backend fails.
/// - [`VctrlError::CorruptedData`] if tree data is malformed during
///   `read_tree`.
pub trait Index {
    /// The type of a single index entry.
    type Entry;

    /// The type used to represent a path in the index.
    type Path;

    /// The type used to identify the tree produced by `write_tree`.
    type TreeId;

    /// Adds an entry to the index.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying index backend cannot be updated.
    fn add(&mut self, entry: Self::Entry) -> Result<(), VctrlError>;

    /// Removes the entry at the given path from the index.
    ///
    /// If the path does not exist, this operation should be a no-op.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying index backend cannot be updated.
    fn remove(&mut self, path: &Self::Path) -> Result<(), VctrlError>;

    /// Removes all entries from the index.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying index backend cannot be cleared.
    fn clear(&mut self) -> Result<(), VctrlError>;

    /// Writes the current index contents into a tree object and returns its ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the tree cannot be created.
    fn write_tree(&self) -> Result<Self::TreeId, VctrlError>;

    /// Clears the index and populates it from the given tree object.
    ///
    /// # Errors
    ///
    /// Returns an error if the tree cannot be read or its contents are
    /// malformed.
    fn read_tree(&mut self, tree: &Self::TreeId) -> Result<(), VctrlError>;
}
