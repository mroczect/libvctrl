//! Builders for the four core object types.
//!
//! This module provides **builder structs** for constructing [`Blob`], [`Tree`],
//! [`Commit`], and [`Tag`] objects in a step‑by‑step, ergonomic way.
//!
//! # Why builders?
//!
//! The fundamental types in `libvctrl_handler` have constructors that accept
//! all required parameters at once. While this is perfectly usable, it can
//! become verbose when some data is only available incrementally (e.g.,
//! building a tree from multiple sources) or when you want to set optional
//! fields (like tagger or message). Builders provide:
//!
//! - **Readability** – each step is labelled with a method name
//!   (`.tree(hash)`, `.author(user)`, …).
//! - **Incremental construction** – you can set fields as data becomes
//!   available, then call `.build()` at the end.
//! - **Validation on finalisation** – the builder defers validation until
//!   `build()` is called, so you can safely manipulate fields without
//!   immediate errors.
//!
//! # Builder lifecycle
//!
//! 1. Create the builder with `::new()`.
//! 2. Call setter methods (`.tree()`, `.author()`, `.entry()`, …).
//!    Each method consumes the builder and returns a new one, enabling
//!    method chaining.
//! 3. Call `.build()` to validate and produce the final object.
//!    This either returns `Ok(object)` or `Err(VctrlError)`.
//!
//! # Validation
//!
//! Builders **do not** duplicate validation logic. They rely entirely on
//! the constructors in `libvctrl_handler` to enforce invariants. This
//! ensures that validation remains the single source of truth. The builders
//! only check that **required fields are present** (and return
//! `VctrlError::Other` if not).
//!
//! # Examples
//!
//! ```rust
//! use libvctrl_core::object::{BlobBuilder, CommitBuilder, TagBuilder, TreeBuilder};
//! use libvctrl_handler::*;
//!
//! // A blob
//! let blob = BlobBuilder::new()
//!     .with_data(b"file contents".to_vec())
//!     .build();
//!
//! // A tree
//! let hash = Hash::from_bytes(&[0xAB; 64]).unwrap();
//! let tree = TreeBuilder::new()
//!     .add_entry("README.md".into(), EntryKind::Blob, hash).unwrap()
//!     .build().unwrap();
//!
//! // A commit
//! let user = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
//! let commit = CommitBuilder::new()
//!     .tree(hash)
//!     .author(user.clone())
//!     .committer(user)
//!     .message("Initial commit")
//!     .build().unwrap();
//!
//! // A tag
//! let tag = TagBuilder::new()
//!     .name("v1.0")
//!     .target(hash)
//!     .message("First release")
//!     .build().unwrap();
//! ```

pub mod blob;
pub mod commit;
pub mod tag;
pub mod tree;

pub use blob::BlobBuilder;
pub use commit::CommitBuilder;
pub use tag::TagBuilder;
pub use tree::{TreeBuilder, TreeEntryBuilder};
