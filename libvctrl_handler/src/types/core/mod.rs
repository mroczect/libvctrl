//! Core data types.

/// Blob type.
pub mod blob;
pub use blob::Blob;

/// Commit type.
pub mod commit;
pub use commit::{Commit, CommitMeta};

/// Hash type.
pub mod hash;
pub use hash::Hash;

/// Tag type.
pub mod tag;
pub use tag::Tag;

/// Tree type.
pub mod tree;
pub use tree::{Tree, TreeEntry};

/// User identifier type.
pub mod user_id;
pub use user_id::UserID;

/// Delta and change types.
pub mod delta;
pub use delta::{ChangeKind, FileDelta, TreeDelta};

/// Reflog entry type.
pub mod reflog;
pub use reflog::ReflogEntry;

/// Merge-related types.
pub mod merge;
pub use merge::{Conflict, MergeResult};
