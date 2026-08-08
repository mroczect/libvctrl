//! # Fundamental Data Types for `libvctrl`
//!
//! This module defines the core data structures that represent the version control
//! objects: **blobs**, **trees**, **commits**, **tags**, and their supporting types
//! like **hashes** and **user identities**.
//!
//! All types in this module are **immutable** by design (once created, their
//! contents cannot be modified) and enforce **strict validation** at construction
//! time. This guarantees that invalid data never enters the system.
//!
//! ## Overview of Types
//!
//! | Type | Description |
//! |------|-------------|
//! | [`Hash`] | A 64‑byte SHA‑512 content identifier |
//! | [`Blob`]  | Raw, uninterpreted file content |
//! | [`Tree`]  | A sorted directory listing of [`TreeEntry`] items |
//! | [`TreeEntry`] | A single entry inside a tree (name, kind, hash) |
//! | [`Commit`] | A snapshot of the repository at a point in time |
//! | [`CommitMeta`] | Metadata for commits and tags (timestamp, timezone, encoding) |
//! | [`Tag`]   | A named pointer (usually to a commit) with optional annotation |
//! | [`UserID`] | Identity of a user (name + email) |
//!
//! ## Common Validation Rules
//!
//! - **Names** (file names, tag names, user names) must be non‑empty and at most
//!   [`MAX_NAME_LENGTH`] (255) bytes.
//! - **Hashes** must be exactly [`HASH_LENGTH`] (64) bytes.
//! - **Trees** require their entries to be **sorted lexicographically** by name
//!   and must not contain duplicate names.
//!
//! These invariants are enforced by the constructors of each type. If validation
//! fails, a [`VctrlError`] is returned.
//!
//! ## Example: Building a Simple Repository Snapshot
//!
//! ```rust
//! # use libvctrl_handler::*;
//! #
//! # // For demonstration, we create a dummy hash from fixed bytes.
//! # let hash = Hash::from_bytes(&[0xAA; HASH_LENGTH]).unwrap();
//! #
//! // 1. Create a blob (file content)
//! let blob_data = b"Hello, world!".to_vec();
//! let blob = Blob::new(blob_data);
//!
//! // 2. Compute its hash (using a hasher from another crate)
//! //    For demonstration, we use a fixed hash.
//! let hash = Hash::from_bytes(&[0xAA; HASH_LENGTH]).unwrap();
//!
//! // 3. Create a tree entry for that blob
//! let entry = TreeEntry::new("hello.txt".into(), EntryKind::Blob, hash).unwrap();
//!
//! // 4. Build a tree (directory) containing that entry
//! let tree = Tree::new(vec![entry]).unwrap();
//! let tree_hash = hash; // In reality, this would be the hash of the encoded tree.
//!
//! // 5. Create an author identity
//! let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
//!
//! // 6. Create a commit pointing to the tree
//! let commit = Commit::new(
//!     tree_hash,
//!     vec![],
//!     author.clone(),      // author
//!     author.clone(),      // committer (clone again)
//!     "Initial commit".into(),
//! );
//!
//! // 7. (Optional) Create an annotated tag for this commit
//! let tag = Tag::new(
//!     "v1.0".into(),
//!     tree_hash,
//!     Some(author.clone()), // tagger
//!     "First release".into(),
//! ).unwrap();
//! # let _ = (blob, commit, tag); // silence unused warnings
//! ```
//!
//! ## Private Helper
//!
//! The function [`validate_name`] is used internally by several constructors to
//! enforce the name length limits. It is not part of the public API.

use crate::constants::MAX_NAME_LENGTH;
use crate::errors::VctrlError;

/// Blob object – raw uninterpreted data.
pub mod blob;
/// Commit object and its metadata.
pub mod commit;
/// Content hash (64‑byte SHA‑512).
pub mod hash;
/// Tag object – named pointer, usually to a commit.
pub mod tag;
/// Tree (directory listing) and tree entries.
pub mod tree;
/// User identity (author / committer).
pub mod user_id;

// Re‑export all public types for easy access.
pub use blob::Blob;
pub use commit::{Commit, CommitMeta};
pub use hash::Hash;
pub use tag::Tag;
pub use tree::{Tree, TreeEntry};
pub use user_id::UserID;

// ---------------------------------------------------------------------------
// Helper for name validation (used by multiple types)
// ---------------------------------------------------------------------------

/// Validates that a name meets the system‑wide constraints.
///
/// This function checks that:
/// - The name is **not empty**.
/// - Its length in bytes does not exceed [`MAX_NAME_LENGTH`] (255).
///
/// It is used internally by constructors of [`TreeEntry`], [`Tag`], [`UserID`],
/// and any other type that requires a human‑readable identifier.
///
/// # Errors
///
/// Returns [`VctrlError::InvalidName`] with a descriptive message if the name
/// is empty or too long.
///
/// This function is **private** to the crate; it is not meant to be used directly
/// by consumers of `libvctrl_handler`.
fn validate_name(name: &str) -> Result<(), VctrlError> {
    if name.is_empty() {
        return Err(VctrlError::InvalidName("name is empty".into()));
    }
    if name.len() > MAX_NAME_LENGTH {
        return Err(VctrlError::InvalidName(format!(
            "name exceeds maximum length {MAX_NAME_LENGTH}: '{name}'"
        )));
    }
    Ok(())
}
