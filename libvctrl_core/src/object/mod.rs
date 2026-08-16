//! Object builders for ergonomic construction of Git objects.

/// Blob builder.
pub mod blob;

/// Commit builder.
pub mod commit;

/// Tag builder.
pub mod tag;

/// Tree builder.
pub mod tree;

pub use blob::BlobBuilder;
pub use commit::CommitBuilder;
pub use tag::TagBuilder;
pub use tree::{TreeBuilder, TreeEntryBuilder};
