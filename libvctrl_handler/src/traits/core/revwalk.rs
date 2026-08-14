//! Defines the `RevWalk` trait for traversing commit graphs.
//!
//! # Purpose
//!
//! The `RevWalk` trait provides the contract for retrieving the parents of a
//! commit, enabling reverse traversal of repository history. This is a
//! fundamental operation for commands like `rev-list`, `merge-base`, and
//! `rev-parse`.
//!
//! # Why a separate module
//!
//! Graph traversal is a distinct responsibility from storage or encoding.
//! Keeping the trait in its own file allows backends to provide different
//! commit graph implementations without affecting other components.
//!
//! # Examples
//!
//! A minimal implementation using [`Hash`] as the commit identifier:
//!
//! ```
//! use libvctrl_handler::{Hash, RevWalk, VctrlError};
//!
//! struct DummyRevWalk;
//!
//! impl RevWalk for DummyRevWalk {
//!     type CommitId = Hash;
//!
//!     fn parents(&self, _id: &Hash) -> Result<Vec<Hash>, VctrlError> {
//!         Ok(vec![])
//!     }
//! }
//!
//! let walker = DummyRevWalk;
//! let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
//! assert!(walker.parents(&hash).unwrap().is_empty());
//! ```

use crate::VctrlError;

/// Trait for traversing the commit graph by retrieving parent commits.
///
/// # Purpose
///
/// `RevWalk` abstracts the ability to query the parents of a given commit,
/// enabling reverse traversal of the repository history. This is a
/// foundational operation for commands such as `rev-list`, `merge-base`,
/// and `rev-parse`. By defining a trait, we allow different backends to
/// provide their own commit graph implementation without coupling the
/// caller to a specific storage engine.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Hash, RevWalk, VctrlError};
///
/// struct DummyRevWalk;
///
/// impl RevWalk for DummyRevWalk {
///     type CommitId = Hash;
///
///     fn parents(&self, _id: &Hash) -> Result<Vec<Hash>, VctrlError> {
///         Ok(vec![])
///     }
/// }
///
/// let walker = DummyRevWalk;
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// assert!(walker.parents(&hash).unwrap().is_empty());
/// ```
///
/// # Errors
///
/// - [`VctrlError::ObjectNotFound`] if the given commit ID does not exist
///   in the underlying repository.
pub trait RevWalk {
    /// The type used to identify a commit (e.g., [`Hash`]).
    type CommitId;

    /// Returns the parent commit IDs of the given commit.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::ObjectNotFound`] if the commit ID is unknown.
    fn parents(&self, id: &Self::CommitId) -> Result<Vec<Self::CommitId>, VctrlError>;
}
