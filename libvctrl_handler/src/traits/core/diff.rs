//! Tree differencing trait.
//!
//! # Architecture
//! This module provides the abstract contract for computing structural deltas
//! between two tree objects. It abstracts the diffing algorithm (e.g., Myers,
//! Histogram) away from the core engine, allowing consumers to plug in
//! optimized or specialized diffing strategies.
//!
//! # Design Rationale: Associated Types over Generics
//! The trait uses an associated type (`type TreeId`) rather than a generic
//! parameter (`<TreeId>`). This design choice is deliberate: it ties the
//! identifier type to the specific `TreeDiffer` implementation. A differ that
//! reads from an in-memory store might use array indices as IDs, while a
//! filesystem-based differ uses `Hash`. Associated types prevent the need to
//! annotate the trait with generics at every call site, simplifying the API
//! while preserving flexibility.

use crate::errors::VctrlError;
use crate::types::TreeDelta;

/// Trait for computing differences between two trees.
///
/// # Why this exists
/// Comparing two trees to find file additions, deletions, modifications, and
/// renames is a fundamental operation in version control. By defining this as
/// a trait, the crate ensures that the core logic does not depend on a specific
/// algorithm or storage backend. The output is a strongly-typed [`TreeDelta`],
/// which aggregates [`FileDelta`](crate::FileDelta) entries, ensuring that
/// downstream consumers (like UI renderers or merge drivers) receive a
/// consistent, validated data structure.
///
/// # How it works
/// The implementor receives references to two tree identifiers (`old` and `new`).
/// It is responsible for resolving these IDs to actual tree data (if necessary),
/// comparing their entries recursively, and classifying the changes. The
/// resulting [`TreeDelta`] provides an iterator-like interface over these
/// atomic file changes.
///
/// # Design Rationale: Thread Safety
/// The trait requires `Send + Sync` on both `Self` and the associated `TreeId`.
/// This is critical for performance: diffing large repositories is highly
/// parallelizable. By enforcing thread safety, the engine can dispatch
/// multiple `diff_trees` calls across a thread pool (e.g., using `rayon`)
/// to compare different directory branches concurrently without data races.
///
/// # Examples
///
/// Implementing the trait for a mock store that always reports no changes:
///
/// ```
/// # use libvctrl_handler::traits::core::diff::TreeDiffer;
/// # use libvctrl_handler::{TreeDelta, Hash, VctrlError};
/// #
/// struct MockDiffer;
///
/// impl TreeDiffer for MockDiffer {
///     type TreeId = Hash;
///
///     fn diff_trees(&self, _old: &Self::TreeId, _new: &Self::TreeId) -> Result<TreeDelta, VctrlError> {
///         // In a real implementation, this would load trees and compare entries.
///         Ok(TreeDelta::new())
///     }
/// }
///
/// let differ = MockDiffer;
/// let old_hash = Hash::from_bytes(&[0_u8; 64])?;
/// let new_hash = Hash::from_bytes(&[1u8; 64])?;
///
/// let delta = differ.diff_trees(&old_hash, &new_hash)?;
/// assert!(delta.is_empty());
/// # Ok::<(), VctrlError>(())
/// ```
pub trait TreeDiffer: Send + Sync {
    /// The identifier type for a tree.
    ///
    /// # Why this exists
    /// Allows the differ implementation to define its own lookup mechanism. While
    /// typically a [`Hash`], it could also be a database primary key or an
    /// in-memory pointer, decoupling the diff logic from the object storage format.
    type TreeId: Send + Sync;

    /// Computes the list of changes between two trees.
    ///
    /// # How it works
    /// Resolves the `old` and `new` identifiers and performs a structural
    /// comparison. The method returns a [`TreeDelta`] containing a list of
    /// [`FileDelta`](crate::FileDelta)s. If a file exists in `new` but not `old`,
    /// it is classified as `Added`; if it exists in `old` but not `new`, it is
    /// `Deleted`. If the hashes differ but paths match, it is `Modified`.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if either tree cannot be loaded (e.g.,
    /// [`VctrlError::ObjectNotFound`]) or if the diffing process fails due to
    /// corrupted data.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::diff::TreeDiffer;
    /// # use libvctrl_handler::{TreeDelta, Hash, VctrlError};
    /// #
    /// # struct MockDiffer;
    /// # impl TreeDiffer for MockDiffer {
    /// #     type TreeId = Hash;
    /// #     fn diff_trees(&self, _old: &Self::TreeId, _new: &Self::TreeId) -> Result<TreeDelta, VctrlError> {
    /// #         Ok(TreeDelta::new())
    /// #     }
    /// # }
    /// let differ = MockDiffer;
    /// let hash = Hash::from_bytes(&[0_u8; 64])?;
    ///
    /// // Diffing a tree against itself should yield an empty delta.
    /// let delta = differ.diff_trees(&hash, &hash)?;
    /// assert_eq!(delta.len(), 0);
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn diff_trees(&self, old: &Self::TreeId, new: &Self::TreeId) -> Result<TreeDelta, VctrlError>;
}
