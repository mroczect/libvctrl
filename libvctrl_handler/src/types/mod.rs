//! Core data types for Git objects.
//!
//! # Architecture
//! This module serves as the central registry for strongly-typed, immutable
//! representations of Git objects and domain concepts. By isolating these data
//! structures into a dedicated `types` module, the crate separates its abstract
//! contracts (in `traits`) from the concrete data carriers used in serialization,
//! manipulation, and network transfer.
//!
//! # Design Rationale: Fallible Construction
//! All types in this module enforce strict invariants during construction (e.g.,
//! [`Hash`] requires exactly 64 bytes, [`Commit`] rejects duplicate parents). By
//! making constructors fallible (returning `Result`), the crate guarantees that
//! invalid states are unrepresentable at runtime. Once constructed, the types are
//! immutable, ensuring thread-safe sharing without external synchronization.
//!
//! # Facade Pattern
//! This module acts as a facade. It delegates the definitions to the `core`
//! submodule and selectively re-exports the public types to the top level. This
//! provides a clean, flat namespace for consumers (e.g., `libvctrl_handler::types::Commit`)
//! while keeping the internal module structure logically separated by domain.

/// Core data type definitions for Git objects and domain concepts.
///
/// # Why this exists
/// Houses the actual struct and enum definitions. Grouping these into a `core`
/// submodule prevents the parent `types` module from becoming a monolithic file,
/// allowing each object type (blob, tree, commit, etc.) to be developed and
/// tested in isolation.
///
/// # Examples
///
/// ```
/// // The core submodule is accessible for advanced or internal use.
/// use libvctrl_handler::types::core;
/// ```
pub mod core;

/// Re-exports of fundamental Git object types for ergonomic, flat access.
///
/// # Why this exists
/// Provides a flattened import path. Consumers can directly use
/// `libvctrl_handler::types::Blob` instead of navigating the full
/// `libvctrl_handler::types::core::blob::Blob` path. This reduces boilerplate in consumer
/// code while keeping the internal module structure logically separated.
///
/// # Examples
///
/// Importing and using multiple core types:
///
/// ```
/// # use libvctrl_handler::types::{Blob, Hash, Tree};
/// # use libvctrl_handler::VctrlError;
/// let raw_bytes = [0_u8; 64];
/// let hash = Hash::from_bytes(&raw_bytes)?;
/// let blob = Blob::new(b"content".to_vec())?;
/// let tree = Tree::new(vec![])?;
///
/// assert_eq!(blob.size(), 7);
/// assert!(tree.is_empty());
/// # Ok::<(), VctrlError>(())
/// ```
pub use core::{
    blob::Blob,
    commit::{Commit, CommitMeta},
    delta::{ChangeKind, FileDelta, TreeDelta},
    hash::Hash,
    merge::{Conflict, MergeResult},
    reflog::ReflogEntry,
    tag::Tag,
    tree::{Tree, TreeEntry},
    user_id::UserID,
};
