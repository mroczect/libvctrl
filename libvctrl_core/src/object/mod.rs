//! Object builders for ergonomic construction of Git objects.
//!
//! # Why this module exists
//!
//! The data types in [`libvctrl_handler`] are immutable and enforce their own
//! invariants through constructors such as
//! [`Commit::new`](libvctrl_handler::Commit::new). While those constructors
//! are safe and correct, they often require every field to be supplied at once.
//! In real applications, fields may arrive gradually from parsing, user input,
//! or configuration. The builder pattern separates gradual assembly from final
//! validation.
//!
//! Each builder in this module consumes `self` on every setter, returns `Self`,
//! and exposes a single `build` method that performs validation and constructs
//! the final object. This design prevents partially configured builders from
//! being used accidentally after construction, while still allowing fluent
//! chains.
//!
//! # Module organization
//!
//! The module mirrors the object type hierarchy:
//!
//! - [`blob`] contains [`BlobBuilder`] for [`Blob`](libvctrl_handler::Blob).
//! - [`tree`] contains [`TreeBuilder`] and [`TreeEntryBuilder`] for
//!   [`Tree`](libvctrl_handler::Tree) and
//!   [`TreeEntry`](libvctrl_handler::TreeEntry).
//! - [`commit`] contains [`CommitBuilder`] for
//!   [`Commit`](libvctrl_handler::Commit).
//! - [`tag`] contains [`TagBuilder`] for [`Tag`](libvctrl_handler::Tag).
//!
//! All builders are re-exported at this module level so callers can use
//! `libvctrl_core::object::CommitBuilder` instead of the longer submodule path.
//!
//! # Examples
//!
//! Construct a commit using the builder:
//!
//! ```
//! use libvctrl_core::object::CommitBuilder;
//! use libvctrl_handler::{Hash, UserID};
//!
//! let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
//! let author = UserID::new("Alice".to_owned(), "alice@example.com".to_owned()).unwrap();
//! let committer = author.clone();
//!
//! let commit = CommitBuilder::new()
//!     .tree(tree)
//!     .author(author)
//!     .committer(committer)
//!     .message("Initial commit")
//!     .build()
//!     .unwrap();
//!
//! assert_eq!(commit.message(), "Initial commit");
//! ```

/// Blob builder.
///
/// This submodule contains [`BlobBuilder`], a builder for constructing
/// [`Blob`](libvctrl_handler::Blob) objects from arbitrary byte data.
pub mod blob;

/// Commit builder.
///
/// This submodule contains [`CommitBuilder`], a builder for constructing
/// [`Commit`](libvctrl_handler::Commit) objects with tree, parents, author,
/// committer, message, and optional metadata.
pub mod commit;

/// Tag builder.
///
/// This submodule contains [`TagBuilder`], a builder for constructing
/// [`Tag`](libvctrl_handler::Tag) objects with a name, target hash, optional
/// tagger, message, and optional metadata.
pub mod tag;

/// Tree builder.
///
/// This submodule contains [`TreeBuilder`] and [`TreeEntryBuilder`], builders
/// for constructing [`Tree`](libvctrl_handler::Tree) and
/// [`TreeEntry`](libvctrl_handler::TreeEntry) objects with sorted entries and
/// entry kinds.
pub mod tree;

/// Re-export of [`BlobBuilder`] for convenient access at the module root.
pub use blob::BlobBuilder;

/// Re-export of [`CommitBuilder`] for convenient access at the module root.
pub use commit::CommitBuilder;

/// Re-export of [`TagBuilder`] for convenient access at the module root.
pub use tag::TagBuilder;

/// Re-export of [`TreeBuilder`] and [`TreeEntryBuilder`] for convenient access
/// at the module root.
pub use tree::{TreeBuilder, TreeEntryBuilder};
