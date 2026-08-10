//! Core data structures for version control objects.
//!
//! # Purpose
//! This module aggregates the fundamental, pure-data types used to represent
//! objects in a version control system. These include [`Blob`] (file contents),
//! [`Tree`] (directory listings), [`Commit`] (history snapshots), and [`Tag`]
//! (named references to commits).
//!
//! # Design rationale
//! The types defined here are intentionally separated from the behavior traits
//! (like [`Encoder`](crate::Encoder) or [`ObjectStore`](crate::ObjectStore)).
//! This separation follows the "data vs. behavior" design pattern:
//! - The structs here are plain data carriers with private fields and getter
//!   methods, ensuring immutability after construction.
//! - The traits in the rest of the crate define *how* these objects are
//!   serialized, stored, and transported.
//!
//! This allows different backends (e.g., in-memory vs. disk-based) to interact
//! with the exact same logical types without coupling the data definitions to
//! I/O logic.
//!
//! # Internal mechanism
//! The module also provides a private `validate_name` helper used by the
//! constructors of name-bearing types ([`Tag`], [`TreeEntry`], [`UserID`]) to
//! enforce length and non-emptiness constraints centrally.

use crate::constants::MAX_NAME_LENGTH;
use crate::errors::VctrlError;

/// Module containing the [`Blob`](crate::Blob) type, representing raw file content.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::types::blob::Blob;
///
/// let blob = Blob::new(vec![0u8; 4]);
/// assert_eq!(blob.size(), 4);
/// ```
pub mod blob;

/// Module containing the [`Commit`](crate::Commit) and [`CommitMeta`](crate::CommitMeta) types, representing historical snapshots.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::types::commit::{Commit, CommitMeta};
///
/// let meta = CommitMeta::default();
/// assert_eq!(meta.timestamp, 0);
/// ```
pub mod commit;

/// Module containing the [`Hash`](crate::Hash) type, a 64-byte cryptographic digest.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::types::hash::Hash;
///
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// assert!(!hash.as_bytes().is_empty());
/// ```
pub mod hash;

/// Module containing the [`Tag`](crate::Tag) type, representing a named reference.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::types::tag::Tag;
/// use libvctrl_handler::Hash;
///
/// let target = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let tag = Tag::new("v1.0".to_string(), target, None, "Release".to_string()).unwrap();
/// assert_eq!(tag.name(), "v1.0");
/// ```
pub mod tag;

/// Module containing the [`Tree`](crate::Tree) and [`TreeEntry`](crate::TreeEntry) types, representing directory structures.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::types::tree::{Tree, TreeEntry};
/// use libvctrl_handler::{EntryKind, Hash};
///
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let entry = TreeEntry::new("file.txt".to_string(), EntryKind::Blob, hash).unwrap();
/// let tree = Tree::new(vec![entry]).unwrap();
/// assert_eq!(tree.entries().len(), 1);
/// ```
pub mod tree;

/// Module containing the [`UserID`](crate::UserID) type, representing identities.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::types::user_id::UserID;
///
/// let user = UserID::new("Alice".to_string(), "alice@example.com".to_string()).unwrap();
/// assert_eq!(user.name(), "Alice");
/// ```
pub mod user_id;

/// Re-export of the [`Blob`](crate::Blob) type for convenience.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::Blob;
///
/// let blob = Blob::new(vec![1, 2, 3]);
/// assert_eq!(blob.size(), 3);
/// ```
pub use blob::Blob;

/// Re-export of the [`Commit`](crate::Commit) and [`CommitMeta`](crate::CommitMeta) types for convenience.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Commit, CommitMeta, Hash, UserID};
///
/// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let author = UserID::new("Alice".to_string(), "alice@example.com".to_string()).unwrap();
/// let committer = UserID::new("Bob".to_string(), "bob@example.com".to_string()).unwrap();
/// let meta = CommitMeta { timestamp: 100, ..Default::default() };
///
/// let commit = Commit::with_meta(tree, Vec::new(), author, committer, "msg".to_string(), meta);
/// assert_eq!(commit.timestamp(), 100);
/// ```
pub use commit::{Commit, CommitMeta};

/// Re-export of the [`Hash`](crate::Hash) type for convenience.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::Hash;
///
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// assert_eq!(hash.as_bytes().len(), 64);
/// ```
pub use hash::Hash;

/// Re-export of the [`Tag`](crate::Tag) type for convenience.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Hash, Tag};
///
/// let target = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let tag = Tag::new("v1.0".to_string(), target, None, "Release".to_string()).unwrap();
/// assert_eq!(tag.name(), "v1.0");
/// ```
pub use tag::Tag;

/// Re-export of the [`Tree`](crate::Tree) and [`TreeEntry`](crate::TreeEntry) types for convenience.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{EntryKind, Hash, Tree, TreeEntry};
///
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let entry = TreeEntry::new("file.txt".to_string(), EntryKind::Blob, hash).unwrap();
/// let tree = Tree::new(vec![entry]).unwrap();
/// assert_eq!(tree.entries().len(), 1);
/// ```
pub use tree::{Tree, TreeEntry};

/// Re-export of the [`UserID`](crate::UserID) type for convenience.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::UserID;
///
/// let user = UserID::new("Alice".to_string(), "alice@example.com".to_string()).unwrap();
/// assert_eq!(user.name(), "Alice");
/// ```
pub use user_id::UserID;

/// Validates a name string according to the system's length and emptiness rules.
///
/// # Why this exists
/// Names in a version control system (e.g., references, tree entries, user names)
/// must be non-empty and bounded in length to prevent resource exhaustion and
/// ensure compatibility with filesystem limits. This helper centralizes the
/// validation logic so that all name-bearing types apply the same rules
/// consistently.
///
/// # How it works
/// It checks if the string slice is empty. If so, it returns an
/// [`InvalidName`](crate::VctrlError::InvalidName) error. Then it checks if the byte
/// length exceeds [`MAX_NAME_LENGTH`]. If it does, it returns an
/// [`InvalidName`](crate::VctrlError::InvalidName) error containing the offending name.
///
/// # Examples
///
/// While this function is private, its behavior is observable through public
/// constructors like [`UserID::new`](crate::UserID::new):
///
/// ```
/// use libvctrl_handler::{UserID, VctrlError};
///
/// // Empty names are rejected
/// let err = UserID::new("".to_string(), "test@example.com".to_string()).unwrap_err();
/// assert!(matches!(err, VctrlError::InvalidName(_)));
///
/// // Names exceeding the max length are rejected
/// let long_name = "a".repeat(libvctrl_handler::MAX_NAME_LENGTH as usize + 1);
/// let err = UserID::new(long_name, "test@example.com".to_string()).unwrap_err();
/// assert!(matches!(err, VctrlError::InvalidName(_)));
/// ```
#[allow(clippy::cast_possible_truncation)]
fn validate_name(name: &str) -> Result<(), VctrlError> {
    if name.is_empty() {
        return Err(VctrlError::InvalidName("name is empty".into()));
    }
    if name.len() > MAX_NAME_LENGTH as usize {
        return Err(VctrlError::InvalidName(format!(
            "name exceeds maximum length {MAX_NAME_LENGTH}: '{name}'"
        )));
    }
    Ok(())
}
