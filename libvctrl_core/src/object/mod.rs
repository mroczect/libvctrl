//! Builder patterns for constructing version control objects.
//!
//! # Purpose
//!
//! This module aggregates the builder implementations for the core version
//! control objects: [`Blob`](libvctrl_handler::Blob),
//! [`Tree`](libvctrl_handler::Tree),
//! [`Commit`](libvctrl_handler::Commit), and
//! [`Tag`](libvctrl_handler::Tag). Builders provide a fluent, ergonomic
//! interface for assembling complex objects step-by-step.
//!
//! # Design Rationale
//!
//! - **Telescoping constructor avoidance**: VCS objects like `Commit` and
//!   `Tag` have many fields, some required and some optional. Using standard
//!   constructors would lead to a combinatorial explosion of `new` methods.
//!   The builder pattern defers validation to a single `build()` method.
//! - **Deferred validation**: Builders accumulate state without performing
//!   heavy validation. When `build()` is called, the final structural
//!   invariants (e.g., tree entries being sorted, required fields being
//!   present) are enforced centrally by the `libvctrl_handler` types.
//! - **Ownership transfer**: The builders consume `self` and return it by
//!   value during configuration. This allows method chaining and ensures that
//!   the underlying data (like `Vec<u8>` or `String`) is moved directly into
//!   the final object with zero heap allocations or cloning overhead.
//! - **Consistency**: All object types use the same builder pattern, making
//!   the construction experience uniform across the crate.
//!
//! # Internal Mechanism
//!
//! Each builder holds intermediate state, typically in `Option` or `Vec`
//! wrappers. When `build()` is invoked, the builder extracts the raw fields,
//! checks for missing required data, and passes them to the constructor of
//! the corresponding [`libvctrl_handler`] type.
//!
//! # Module Structure
//!
//! The module is organized into the following submodules:
//!
//! - [`blob`](self::blob): [`BlobBuilder`] for constructing blobs.
//! - [`commit`](self::commit): [`CommitBuilder`] for constructing commits.
//! - [`tag`](self::tag): [`TagBuilder`] for constructing tags.
//! - [`tree`](self::tree): [`TreeBuilder`] and [`TreeEntryBuilder`] for
//!   constructing trees and tree entries.
//!
//! Each builder is re-exported at the module root for ergonomic access.
//!
//! # How Builders Relate to Handlers
//!
//! The builders do not replace the constructors provided by
//! `libvctrl_handler`. Instead, they provide a more readable and flexible
//! way to collect values and then delegate to the native constructors. This
//! keeps the handler types pure and immutable while moving the assembly
//! logic to `libvctrl_core`.
//!
//! # Examples
//!
//! Building a commit with the fluent API:
//!
//! ```
//! use libvctrl_core::object::CommitBuilder;
//! use libvctrl_handler::{Hash, UserID};
//!
//! let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
//! let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
//! let committer = UserID::new("Bob".into(), "bob@example.com".into()).unwrap();
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

/// Module containing the [`BlobBuilder`](crate::object::BlobBuilder)
/// implementation.
///
/// # Purpose
///
/// Provides a fluent API for constructing [`Blob`](libvctrl_handler::Blob)
/// objects.
///
/// # Design Rationale
///
/// The builder is isolated in its own module to keep the object module
/// organized and to allow future extensions such as optional compression or
/// validation before finalization.
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

/// Module containing the [`CommitBuilder`](crate::object::CommitBuilder)
/// implementation.
///
/// # Purpose
///
/// Provides a fluent API for constructing [`Commit`](libvctrl_handler::Commit)
/// objects.
///
/// # Design Rationale
///
/// Commits have several required and optional fields. The builder pattern
/// avoids telescoping constructors and centralizes missing-field validation
/// in the `build()` method.
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

/// Module containing the [`TagBuilder`](crate::object::TagBuilder)
/// implementation.
///
/// # Purpose
///
/// Provides a fluent API for constructing [`Tag`](libvctrl_handler::Tag)
/// objects.
///
/// # Design Rationale
///
/// Tags have optional tagger and message fields, making the builder pattern
/// ideal for supporting both lightweight and annotated tags.
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
///
/// Provides fluent APIs for constructing [`Tree`](libvctrl_handler::Tree)
/// and [`TreeEntry`](libvctrl_handler::TreeEntry) objects.
///
/// # Design Rationale
///
/// Trees require entries to be sorted and validated. The builders allow
/// incremental assembly and delegate final structural validation to
/// [`Tree::new`](libvctrl_handler::Tree::new).
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
///
/// Flattens the module path so users can simply import
/// `libvctrl_core::object::BlobBuilder` instead of the full path.
///
/// # Design Rationale
///
/// Re-exporting at the module root reduces boilerplate and improves the
/// ergonomic experience for consumers of the crate.
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
///
/// Flattens the module path so users can simply import
/// `libvctrl_core::object::CommitBuilder`.
///
/// # Design Rationale
///
/// Re-exporting provides a clean public API surface while maintaining
/// internal modularity.
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
///
/// Flattens the module path so users can simply import
/// `libvctrl_core::object::TagBuilder`.
///
/// # Design Rationale
///
/// Re-exporting keeps the API ergonomic and consistent with the rest of the
/// crate.
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
///
/// Flattens the module path so users can simply import
/// `libvctrl_core::object::TreeBuilder` and `TreeEntryBuilder`.
///
/// # Design Rationale
///
/// Re-exporting both tree builders together simplifies imports and keeps the
/// public API clean.
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
