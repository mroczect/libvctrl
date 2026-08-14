//! # Core Types
//!
//! The foundational data structures of the version control system.
//!
//! # Purpose
//!
//! This module contains the canonical definitions of all persistent objects:
//! `Blob`, `Commit`, `CommitMeta`, `Hash`, `Tag`, `Tree`,
//! `TreeEntry`, and `UserID`. Each type is defined in its own submodule
//! and re-exported here for direct access. These types form the immutable
//! core of the content-addressable object model used throughout
//! `libvctrl_handler`.
//!
//! # Architecture
//!
//! - **Separation of concerns**: Each type lives in a separate file to keep
//!   compilation units small and dependencies explicit. This makes the code
//!   easier to navigate and maintain.
//! - **Re-exports**: `pub use` statements lift the types into the `core`
//!   namespace, allowing `use libvctrl_handler::types::core::Blob` instead
//!   of the deeper `libvctrl_handler::types::core::blob::Blob`. This keeps
//!   the public API clean while preserving internal modularity.
//! - **Immutability by default**: All types are designed with private fields
//!   and public constructors, ensuring invariants are maintained at creation
//!   time. Once an object is constructed, its fields cannot be mutated
//!   directly, which is essential for content addressing.
//!
//! # How the Types Relate
//!
//! - A `Commit` points to a `Tree` via its root tree hash, and
//!   optionally to parent commits via parent hashes. It records who authored
//!   and committed the change, along with a message and metadata.
//! - A `Tree` contains a sorted list of `TreeEntry` items. Each entry
//!   associates a name with a `Hash` and an `EntryKind`
//!   that indicates whether the entry points to a blob or another tree.
//! - A `Blob` represents raw file content and is referred to by a `Hash`
//!   stored in a tree entry. Blobs are the leaves of the object graph.
//! - A `Tag` provides a human-readable name (e.g., a release version) for
//!   a commit, tree, or blob, often with an optional tagger and message.
//! - `UserID` captures the identity of an actor (name + email) and is used
//!   in commits and tags to record authorship.
//! - `CommitMeta` holds optional timestamp, timezone offset, and encoding
//!   information shared by commits and tags.
//! - `Hash` is the content address that identifies every object. It is a
//!   64-byte cryptographic digest.
//!
//! # Design Rationale
//!
//! The object model follows the principles of content-addressable storage:
//!
//! 1. **Identity = content**: An object's hash is derived from its bytes.
//!    Therefore objects are immutable; any modification creates a new hash
//!    and thus a new object.
//! 2. **Hierarchy through references**: Trees reference blobs and other
//!    trees by hash, forming a Merkle DAG. Commits reference trees and
//!    parent commits. Tags reference arbitrary objects.
//! 3. **Validation at boundaries**: Constructors validate inputs (name
//!    length, hash length, tree entry ordering, etc.) and return `Result` to
//!    prevent malformed objects from entering the system.
//!
//! # Internal Note
//!
//! The module is intentionally free of logic beyond type definitions and
//! accessors. Behaviour such as serialization, hashing, and storage is
//! defined in the `traits` module. This separation keeps
//! the data model pure and easy to reason about.
//!
//! # Examples
//!
//! Importing core types through this module:
//!
//! ```
//! use libvctrl_handler::types::core::{Blob, Hash};
//!
//! let blob = Blob::new(b"sample content".to_vec());
//! let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
//!
//! assert_eq!(blob.size(), 14);
//! assert_eq!(hash.as_bytes().len(), 64);
//! ```
pub mod blob;

/// Binary Large Object.
///
/// # Purpose
///
/// A `Blob` stores arbitrary byte content exactly as provided. It is the
/// simplest object in the system and is content-addressable via its `Hash`.
/// Blobs represent file contents in a version control tree.
///
/// # Design Rationale
///
/// - The blob owns its data (`Vec<u8>`) to avoid lifetime parameters and
///   allow easy cloning and moving.
/// - No interpretation is performed on the bytes; the blob is a pure data
///   container.
/// - Access is read-only, preserving immutability.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::types::core::Blob;
///
/// let blob = Blob::new(b"hello, world".to_vec());
/// assert_eq!(blob.size(), 12);
/// assert!(!blob.is_empty());
/// ```
pub use blob::Blob;

pub mod commit;

/// Commit metadata and the commit object itself.
///
/// # Purpose
///
/// `Commit` records a snapshot of the repository state. It carries a
/// pointer to the root tree, references to parent commits, author/committer
/// information, a message, and optional metadata.
///
/// `CommitMeta` is a separate struct that holds timestamp, timezone
/// offset, and encoding information, allowing the commit to be constructed
/// with or without explicit metadata.
///
/// # Design Rationale
///
/// - The commit is immutable after construction, preserving its hash.
/// - The separation of `Commit` and `CommitMeta` keeps the main struct
///   uncluttered and allows default values when metadata is absent.
/// - The commit message and other fields are private with accessor methods
///   to enforce invariants.
///
/// # Examples
///
/// Building a commit with metadata:
///
/// ```
/// use libvctrl_handler::types::core::{Commit, CommitMeta, Hash, UserID};
///
/// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let author = UserID::new("Author".into(), "author@example.com".into()).unwrap();
/// let committer = UserID::new("Committer".into(), "committer@example.com".into()).unwrap();
/// let meta = CommitMeta {
///     timestamp: 1_700_000_000,
///     timezone_offset: 360,
///     encoding: Some("utf-8".into()),
/// };
///
/// let commit = Commit::with_meta(
///     tree,
///     vec![],
///     author,
///     committer,
///     "Initial commit".into(),
///     meta,
/// );
///
/// assert_eq!(commit.message(), "Initial commit");
/// assert_eq!(commit.parents().len(), 0);
/// assert_eq!(commit.timestamp(), 1_700_000_000);
/// ```
///
/// For brevity, real construction may also use `Commit::new` when metadata
/// is not needed.
pub use commit::{Commit, CommitMeta};

pub mod hash;

/// Content-addressable hash value.
///
/// # Purpose
///
/// `Hash` is a fixed-size cryptographic digest (64 bytes) that identifies
/// objects. It is used throughout the system for deduplication and integrity
/// verification.
///
/// # Design Rationale
///
/// - The hash is stored as a byte array of length
///   `HASH_LENGTH`, ensuring stack
///   allocation and cheap copies.
/// - The type implements `Copy`, `Eq`, `Ord`, and `Hash`, making it suitable
///   as a key in maps and sets.
/// - Construction from a slice is fallible to enforce the exact length
///   invariant.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::types::core::Hash;
///
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// assert_eq!(hash.as_bytes().len(), 64);
/// ```
pub use hash::Hash;

pub mod tag;

/// Named reference to an object (usually a commit).
///
/// # Purpose
///
/// A `Tag` associates a human-readable name with a specific `Hash` and
/// optional metadata (tagger, message, timestamp). Tags are typically used
/// to mark releases or significant points in history.
///
/// # Design Rationale
///
/// - The tag name is validated at construction to enforce length and
///   non-emptiness.
/// - The optional tagger and message support both lightweight and annotated
///   tags.
/// - Reuses `CommitMeta` for timestamp and encoding information, avoiding
///   duplication.
///
/// # Examples
///
/// Creating a lightweight tag:
///
/// ```
/// use libvctrl_handler::types::core::{Hash, Tag};
///
/// let target = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let tag = Tag::new("v1.0.0".into(), target, None, "".into()).unwrap();
/// assert_eq!(tag.name(), "v1.0.0");
/// ```
pub use tag::Tag;

pub mod tree;

/// Directory snapshot represented as a list of entries.
///
/// # Purpose
///
/// A `Tree` contains a sorted list of `TreeEntry` items, each linking a
/// name, an `EntryKind`, and a `Hash`. Trees reference
/// both blobs (files) and other trees (subdirectories), forming the
/// hierarchical structure of a repository.
///
/// # Design Rationale
///
/// - Entries are sorted by name to ensure deterministic serialization and
///   hashing.
/// - The tree is immutable; its entries cannot be modified after creation.
/// - Validation at construction prevents duplicate or unsorted entries.
///
/// # Examples
///
/// Creating an empty tree:
///
/// ```
/// use libvctrl_handler::types::core::Tree;
///
/// let tree = Tree::new(vec![]).unwrap();
/// assert!(tree.entries().is_empty());
/// ```
pub use tree::{Tree, TreeEntry};

pub mod user_id;

/// Identity of a user (name and email).
///
/// # Purpose
///
/// `UserID` is used to record who authored or committed a change. It
/// enforces non-empty name and valid email format at construction.
///
/// # Design Rationale
///
/// - Validation is performed in the constructor to guarantee a well-formed
///   identity.
/// - Fields are private to prevent accidental mutation.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::types::core::UserID;
///
/// let user = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
/// assert_eq!(user.name(), "Alice");
/// assert_eq!(user.email(), "alice@example.com");
/// ```
pub use user_id::UserID;
