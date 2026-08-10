//! Builder patterns for constructing version control objects.
//!
//! # Purpose
//! This module aggregates the builder implementations for the core version
//! control objects: [`Blob`](libvctrl_handler::Blob), [`Tree`](libvctrl_handler::Tree),
//! [`Commit`](libvctrl_handler::Commit), and [`Tag`](libvctrl_handler::Tag).
//! Builders provide a fluent, ergonomic interface for assembling complex objects
//! step-by-step.
//!
//! # Design rationale
//! - **Telescoping Constructor Avoidance**: VCS objects like `Commit` and `Tag`
//!   have many fields, some required and some optional. Using standard
//!   constructors would lead to a combinatorial explosion of `new` methods.
//!   The builder pattern defers validation to a single `build()` method.
//! - **Deferred Validation**: Builders accumulate state without performing
//!   heavy validation. When `build()` is called, the final structural invariants
//!   (e.g., tree entries being sorted) are enforced centrally by the
//!   [`libvctrl_handler`] types.
//! - **Ownership Transfer**: The builders consume `self` and return it by value
//!   during configuration. This allows method chaining and ensures that the
//!   underlying data (like `Vec<u8>` or `String`) is moved directly into the
//!   final object with zero heap allocations or cloning overhead.
//!
//! # Internal mechanism
//! Each builder holds intermediate state (usually `Option` or `Vec` wrappers).
//! When [`build`](libvctrl_core::object::CommitBuilder::build) is invoked, the
//! builder extracts the raw fields, checks for missing required data, and passes
//! them to the constructor of the corresponding [`libvctrl_handler`] type.

/// Module containing the [`BlobBuilder`](crate::object::BlobBuilder) implementation.
///
/// # Purpose
/// Provides a fluent API for constructing [`Blob`](libvctrl_handler::Blob) objects.
///
/// # Examples
///
/// ```
/// use libvctrl_core::object::blob::BlobBuilder;
///
/// let blob = BlobBuilder::new()
///     .with_data(b"hello".to_vec())
///     .build();
///
/// assert_eq!(blob.size(), 5);
/// ```
pub mod blob;

/// Module containing the [`CommitBuilder`](crate::object::CommitBuilder) implementation.
///
/// # Purpose
/// Provides a fluent API for constructing [`Commit`](libvctrl_handler::Commit) objects.
///
/// # Examples
///
/// ```
/// use libvctrl_core::object::commit::CommitBuilder;
/// use libvctrl_handler::{Hash, UserID};
///
/// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let user = UserID::new("Alice".to_string(), "a@b.com".to_string()).unwrap();
///
/// let commit = CommitBuilder::new()
///     .tree(tree)
///     .author(user.clone())
///     .committer(user)
///     .message("Initial commit")
///     .build()
///     .unwrap();
///
/// assert_eq!(commit.message(), "Initial commit");
/// ```
pub mod commit;

/// Module containing the [`TagBuilder`](crate::object::TagBuilder) implementation.
///
/// # Purpose
/// Provides a fluent API for constructing [`Tag`](libvctrl_handler::Tag) objects.
///
/// # Examples
///
/// ```
/// use libvctrl_core::object::tag::TagBuilder;
/// use libvctrl_handler::Hash;
///
/// let target = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let tag = TagBuilder::new()
///     .name("v1.0")
///     .target(target)
///     .build()
///     .unwrap();
///
/// assert_eq!(tag.name(), "v1.0");
/// ```
pub mod tag;

/// Module containing the [`TreeBuilder`](crate::object::TreeBuilder) and
/// [`TreeEntryBuilder`](crate::object::TreeEntryBuilder) implementations.
///
/// # Purpose
/// Provides fluent APIs for constructing [`Tree`](libvctrl_handler::Tree) and
/// [`TreeEntry`](libvctrl_handler::TreeEntry) objects.
///
/// # Examples
///
/// ```
/// use libvctrl_core::object::tree::TreeBuilder;
/// use libvctrl_handler::{EntryKind, Hash};
///
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let tree = TreeBuilder::new()
///     .add_entry("file.txt".to_string(), EntryKind::Blob, hash)?
///     .build()
///     .unwrap();
///
/// assert_eq!(tree.entries().len(), 1);
/// # Ok::<(), libvctrl_handler::VctrlError>(())
/// ```
pub mod tree;

/// Re-export of the [`BlobBuilder`](crate::object::BlobBuilder) struct.
///
/// # Purpose
/// Flattens the module path so users can simply import
/// `libvctrl_core::object::BlobBuilder`.
///
/// # Examples
///
/// ```
/// use libvctrl_core::object::BlobBuilder;
///
/// let blob = BlobBuilder::default().build();
/// assert!(blob.is_empty());
/// ```
pub use blob::BlobBuilder;

/// Re-export of the [`CommitBuilder`](crate::object::CommitBuilder) struct.
///
/// # Purpose
/// Flattens the module path so users can simply import
/// `libvctrl_core::object::CommitBuilder`.
///
/// # Examples
///
/// ```
/// use libvctrl_core::object::CommitBuilder;
/// use libvctrl_handler::{Hash, UserID};
///
/// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let user = UserID::new("A".to_string(), "a@a.com".to_string()).unwrap();
///
/// let commit = CommitBuilder::new()
///     .tree(tree)
///     .author(user.clone())
///     .committer(user)
///     .message("msg")
///     .build()
///     .unwrap();
///
/// assert_eq!(commit.parents().len(), 0);
/// ```
pub use commit::CommitBuilder;

/// Re-export of the [`TagBuilder`](crate::object::TagBuilder) struct.
///
/// # Purpose
/// Flattens the module path so users can simply import
/// `libvctrl_core::object::TagBuilder`.
///
/// # Examples
///
/// ```
/// use libvctrl_core::object::TagBuilder;
/// use libvctrl_handler::Hash;
///
/// let target = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let tag = TagBuilder::new()
///     .name("v2.0")
///     .target(target)
///     .build()
///     .unwrap();
///
/// assert_eq!(tag.name(), "v2.0");
/// ```
pub use tag::TagBuilder;

/// Re-export of the [`TreeBuilder`](crate::object::TreeBuilder) and
/// [`TreeEntryBuilder`](crate::object::TreeEntryBuilder) structs.
///
/// # Purpose
/// Flattens the module path so users can simply import
/// `libvctrl_core::object::TreeBuilder` and `TreeEntryBuilder`.
///
/// # Examples
///
/// ```
/// use libvctrl_core::object::{TreeBuilder, TreeEntryBuilder};
/// use libvctrl_handler::{EntryKind, Hash};
///
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let entry = TreeEntryBuilder::new("dir".to_string(), EntryKind::Tree, hash)
///     .build()
///     .unwrap();
///
/// let tree = TreeBuilder::new()
///     .entry(entry)
///     .build()
///     .unwrap();
///
/// assert_eq!(tree.entries().len(), 1);
/// ```
pub use tree::{TreeBuilder, TreeEntryBuilder};
