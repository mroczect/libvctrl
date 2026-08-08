//! Builders for the four core object types.
//!
//! Each builder is a thin, ergonomic wrapper around the corresponding
//! constructor in [`libvctrl_handler`]. They do not duplicate validation;
//! they simply forward to the fundamental constructors.

pub mod blob;
pub mod commit;
pub mod tag;
pub mod tree;

pub use blob::BlobBuilder;
pub use commit::CommitBuilder;
pub use tag::TagBuilder;
pub use tree::{TreeBuilder, TreeEntryBuilder};
