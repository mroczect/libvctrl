//! Defines the `TreeDiffer` trait for comparing tree objects.
//!
//! # Purpose
//!
//! The `TreeDiffer` trait provides a contract for diffing two tree
//! objects and producing a list of changes (additions, deletions,
//! modifications). This abstraction is used by plumbing commands
//! such as `diff-tree`, `diff-index`, and `merge-trees`, and it
//! allows different diff algorithms to be plugged in without
//! coupling the caller to a specific implementation.
//!
//! # Why a separate module
//!
//! Tree diffing is a distinct responsibility from storage, encoding,
//! or traversal. Keeping the trait in its own file isolates the
//! diffing contract, making it easier to maintain and evolve.
//!
//! # Examples
//!
//! A minimal implementation using [`Hash`] as tree ID and `String`
//! as path type:
//!
//! ```
//! use libvctrl_handler::{Change, Hash, TreeDiffer, VctrlError};
//!
//! struct DummyTreeDiffer;
//!
//! impl TreeDiffer for DummyTreeDiffer {
//!     type TreeId = Hash;
//!     type Path = String;
//!
//!     fn diff_trees(
//!         &self,
//!         _old: &Self::TreeId,
//!         _new: &Self::TreeId,
//!     ) -> Result<Vec<Change>, VctrlError> {
//!         Ok(vec![])
//!     }
//! }
//!
//! let differ = DummyTreeDiffer;
//! let old = Hash::from_bytes(&[0u8; 64]).unwrap();
//! let new = Hash::from_bytes(&[1u8; 64]).unwrap();
//! assert!(differ.diff_trees(&old, &new).unwrap().is_empty());
//! ```

use crate::VctrlError;

/// A single change detected between two trees.
///
/// This enum is intentionally minimal and will likely be replaced or
/// extended by richer types (`TreeDelta`, `FileDelta`) in a future
/// issue. For now it captures the three fundamental change categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Change {
    /// A path was added in the new tree.
    Added,
    /// A path was deleted from the old tree.
    Deleted,
    /// A path existed in both trees but its content changed.
    Modified,
}

/// Trait for comparing two tree objects and producing a list of changes.
///
/// # Purpose
///
/// `TreeDiffer` abstracts the ability to compute the difference between
/// two tree objects, yielding a collection of [`Change`] items. This is
/// the foundation for tree-based diff commands and merge operations.
///
/// # Associated Types
///
/// - `TreeId`: the type used to identify a tree (e.g., [`Hash`]).
/// - `Path`: the type used to represent a filesystem path within a tree.
///
/// # Examples
///
/// A trivial implementation that always returns an empty list:
///
/// ```
/// use libvctrl_handler::{Change, Hash, TreeDiffer, VctrlError};
///
/// struct EmptyTreeDiffer;
///
/// impl TreeDiffer for EmptyTreeDiffer {
///     type TreeId = Hash;
///     type Path = String;
///
///     fn diff_trees(
///         &self,
///         _old: &Self::TreeId,
///         _new: &Self::TreeId,
///     ) -> Result<Vec<Change>, VctrlError> {
///         Ok(vec![])
///     }
/// }
///
/// let differ = EmptyTreeDiffer;
/// let old = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let new = Hash::from_bytes(&[1u8; 64]).unwrap();
/// assert!(differ.diff_trees(&old, &new).unwrap().is_empty());
/// ```
///
/// # Errors
///
/// - [`VctrlError::ObjectNotFound`] if either the `old` or `new` tree
///   ID does not exist.
/// - [`VctrlError::CorruptedData`] if the tree data is malformed.
pub trait TreeDiffer {
    /// The type used to identify a tree object.
    type TreeId;

    /// The type used to represent a path inside the tree.
    type Path;

    /// Compares two trees and returns the list of changes.
    ///
    /// # Errors
    ///
    /// Returns an error if either tree cannot be found or if its
    /// contents cannot be decoded.
    fn diff_trees(&self, old: &Self::TreeId, new: &Self::TreeId)
    -> Result<Vec<Change>, VctrlError>;
}
