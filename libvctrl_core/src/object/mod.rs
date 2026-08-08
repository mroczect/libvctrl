//! let user = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
pub mod blob;
pub mod commit;
pub mod tag;
pub mod tree;

pub use blob::BlobBuilder;
pub use commit::CommitBuilder;
pub use tag::TagBuilder;
pub use tree::{TreeBuilder, TreeEntryBuilder};
