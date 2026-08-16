//! Core data types for Git objects.

/// Blob object representation.
pub mod blob;
pub use blob::Blob;

/// Commit object and metadata representation.
pub mod commit;
pub use commit::{Commit, CommitMeta};

/// Delta and change types.
pub mod delta;
pub use delta::{ChangeKind, FileDelta, TreeDelta};

/// Hash type.
pub mod hash;
pub use hash::Hash;

/// Merge-related types.
pub mod merge;
pub use merge::{Conflict, MergeResult};

/// Reflog entry type.
pub mod reflog;
pub use reflog::ReflogEntry;

/// Tag object representation.
pub mod tag;
pub use tag::Tag;

/// Tree object and entry representation.
pub mod tree;
pub use tree::{Tree, TreeEntry};

/// User identity representation.
pub mod user_id;
pub use user_id::UserID;
