//! Core data types for Git objects.
//!
//! # Architecture
//! This module aggregates the fundamental, strongly-typed data structures that
//! represent the Git object model. By separating these types into their own
//! submodules (e.g., `blob`, `commit`, `tree`), the crate prevents the formation
//! of a monolithic, unmanageable file. Each submodule encapsulates the specific
//! validation logic and invariants for its domain.
//!
//! # Design Rationale: Immutable Domain Model
//! All types exported from this module are immutable once constructed. Their
//! constructors are fallible (`Result`-returning), enforcing strict invariants
//! such as hash lengths, maximum sizes, and structural integrity (e.g., sorted
//! tree entries). This guarantees that if an object exists in memory, it is
//! structurally valid and safe to share across threads without external
//! synchronization.
//!
//! # Facade Re-exports
//! While definitions live in submodules, the types are re-exported directly here.
//! This allows consumers to use ergonomic paths like `my_crate::types::core::Blob`
//! instead of the deeper `my_crate::types::core::blob::Blob`.
//!
//! # Examples
//! *Note: The following examples assume this crate is named `my_crate`.*
//!
//! ```
//! # use my_crate::types::core::{Blob, Hash, Tree};
//! # use my_crate::VctrlError;
//! let raw_bytes = [0u8; 64];
//! let hash = Hash::from_bytes(&raw_bytes)?;
//! let blob = Blob::new(b"content".to_vec())?;
//! let tree = Tree::new(vec![])?;
//!
//! assert_eq!(blob.size(), 7);
//! assert!(tree.is_empty());
//! # Ok::<(), VctrlError>(())
//! ```

/// Blob object representation.
///
/// # Why this exists
/// Git blobs represent the raw content of files. This submodule houses the
/// [`Blob`](blob::Blob) type, which enforces size limits during construction
/// to prevent memory exhaustion.
pub mod blob;
pub use blob::Blob;

/// Commit object and metadata representation.
///
/// # Why this exists
/// Commits link tree states together in a directed acyclic graph (DAG). This
/// submodule houses [`Commit`](commit::Commit) and [`CommitMeta`](commit::CommitMeta),
/// enforcing rules like maximum parent counts and duplicate parent detection.
pub mod commit;
pub use commit::{Commit, CommitMeta};

/// Delta and change types.
///
/// # Why this exists
/// Represents structural differences between trees without loading entire file
/// contents. Contains [`ChangeKind`](delta::ChangeKind), [`FileDelta`](delta::FileDelta),
/// and [`TreeDelta`](delta::TreeDelta).
pub mod delta;
pub use delta::{ChangeKind, FileDelta, TreeDelta};

/// Hash type.
///
/// # Why this exists
/// Provides a stack-allocated, `Copy` wrapper for 64-byte SHA-512 hashes via the
/// [`Hash`](hash::Hash) type, eliminating heap allocations for object identifiers.
pub mod hash;
pub use hash::Hash;

/// Merge-related types.
///
/// # Why this exists
/// Represents the outcome of a 3-way merge operation. Contains
/// [`Conflict`](merge::Conflict) and [`MergeResult`](merge::MergeResult).
pub mod merge;
pub use merge::{Conflict, MergeResult};

/// Reflog entry type.
///
/// # Why this exists
/// Represents a single timestamped mutation in the reference history via the
/// [`ReflogEntry`](reflog::ReflogEntry) type.
pub mod reflog;
pub use reflog::ReflogEntry;

/// Tag object representation.
///
/// # Why this exists
/// Annotated tags point to other objects (usually commits) and carry their own
/// metadata. This submodule houses the [`Tag`](tag::Tag) type.
pub mod tag;
pub use tag::Tag;

/// Tree object and entry representation.
///
/// # Why this exists
/// Trees represent the directory structure, mapping names to modes and hashes.
/// This submodule houses [`Tree`](tree::Tree) and [`TreeEntry`](tree::TreeEntry),
/// enforcing Git's strict sorting and duplication rules.
pub mod tree;
pub use tree::{Tree, TreeEntry};

/// User identity representation.
///
/// # Why this exists
/// Represents the `Name <email>` syntax used in commits and tags via the
/// [`UserID`](user_id::UserID) type.
pub mod user_id;
pub use user_id::UserID;
