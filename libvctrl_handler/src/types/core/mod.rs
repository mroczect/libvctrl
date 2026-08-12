//! # Core Types
//!
//! The foundational data structures of the version control system.
//!
//! This module contains the canonical definitions of all persistent objects: [`Blob`],
//! [`Commit`], [`CommitMeta`], [`Hash`], [`Tag`], [`Tree`], [`TreeEntry`], and [`UserID`].
//! Each type is defined in its own submodule and re-exported here for direct access.
//!
//! ## Architecture
//!
//! - **Separation of concerns**: Each type lives in a separate file to keep compilation
//!   units small and dependencies explicit.
//! - **Re-exports**: `pub use` statements lift the types into the `core` namespace,
//!   allowing `use libvctrl_handler::types::core::Blob` instead of the deeper
//!   `libvctrl_handler::types::core::blob::Blob`.
//! - **Immutability by default**: All types are designed with private fields and public
//!   constructors, ensuring invariants are maintained at creation time.
//!
//! ## How the types relate
//!
//! - A [`Commit`] points to a [`Tree`] via its `tree_hash` field, and optionally to
//!   parent commits via `parent_hashes`.
//! - A [`Tree`] contains [`TreeEntry`] items, each associating a name with a [`Hash`].
//! - A [`Blob`] represents raw file content and is referred to by a [`Hash`] stored in a
//!   tree entry.
//! - A [`Tag`] provides a human-readable name for a commit, tree, or blob.
//! - [`UserID`] captures the author/committer identity (name + email).
//!
//! # Examples
//!
//! Importing core types through this module:
//!
//! ```
//! use libvctrl_handler::types::core::Blob;
//! use libvctrl_handler::types::core::Commit;
//! use libvctrl_handler::types::core::Hash;
//!
//! let blob = Blob::new(b"sample content".to_vec());
//! let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
//! ```

pub mod blob;

/// Binary Large Object.
///
/// A [`Blob`] stores arbitrary byte content exactly as provided. It is the
/// simplest object in the system and is content-addressable via its [`Hash`].
///
/// # Examples
///
/// ```
/// use libvctrl_handler::types::core::Blob;
/// let blob = Blob::new(b"hello, world".to_vec());
/// ```
pub use blob::Blob;

pub mod commit;

/// Commit metadata and the commit object itself.
///
/// [`Commit`] records a snapshot of the repository state. It carries a pointer
/// to the root tree, references to parent commits, author/committer
/// information, and a message.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::types::core::{Commit, CommitMeta, Hash, UserID};
/// # use std::str::FromStr;
/// # let author = UserID::new("Author".into(), "author@example.com".into()).unwrap();
/// # let meta = CommitMeta {
/// #     timestamp: 1_700_000_000,
/// #     timezone_offset: 360,
/// #     encoding: Some("utf-8".into()),
/// # };
/// # let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let commit = Commit::with_meta(tree, vec![], author.clone(), author.clone(), "Initial commit".into(), meta);
/// assert_eq!(commit.message(), "Initial commit");
/// assert_eq!(commit.parents().len(), 0);
/// ```
/// For brevity, real construction will use `Commit::new` or a builder.
pub use commit::{Commit, CommitMeta};

pub mod hash;

/// Content-addressable hash value.
///
/// [`Hash`] is a fixed-size cryptographic digest (e.g., SHA-256) that
/// identifies objects. It is used throughout the system for deduplication
/// and integrity verification.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::types::core::Hash;
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// ```
pub use hash::Hash;

pub mod tag;

/// Named reference to an object (usually a commit).
///
/// A [`Tag`] associates a human-readable name with a specific [`Hash`] and
/// optional metadata (tagger, message).
///
/// # Examples
///
/// ```
/// use libvctrl_handler::types::core::Tag;
/// use libvctrl_handler::types::core::Hash;
/// # let dummy_hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let tag = Tag::new("v1.0.0".into(), dummy_hash, None, "".into()).unwrap();
/// let tag = Tag::new("v1.0.0".into(), dummy_hash, None, "".into()).unwrap();
/// ```
pub use tag::Tag;

pub mod tree;

/// Directory snapshot represented as a list of entries.
///
/// A [`Tree`] contains a sorted list of [`TreeEntry`] items, each linking a
/// name and a [`Hash`]. Trees reference both blobs (files) and other trees
/// (subdirectories).
///
/// # Examples
///
/// ```
/// use libvctrl_handler::types::core::{Tree, TreeEntry};
/// # // Minimal construction; actual usage may require builder
/// let tree = Tree::new(vec![]);
/// ```
pub use tree::{Tree, TreeEntry};

pub mod user_id;

/// Identity of a user (name and email).
///
/// [`UserID`] is used to record who authored or committed a change.
/// It enforces non-empty name and valid email format at construction.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::types::core::UserID;
/// let user = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
/// ```
pub use user_id::UserID;
