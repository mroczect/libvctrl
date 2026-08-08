//! # Trees – Directory Listings
//!
//! A `Tree` represents a directory (folder) in the version‑controlled repository.
//! It contains a sorted list of entries, each mapping a **name** to either a
//! **blob** (file) or another **tree** (subdirectory). Trees form the hierarchical
//! structure that gives meaning to blobs.
//!
//! ## Structure
//!
//! ```text
//! Tree (directory)
//! ├── "README.md" → Blob (file content)
//! ├── "src/"      → Tree (subdirectory)
//! │   ├── "main.rs" → Blob
//! │   └── "lib.rs"  → Blob
//! └── "Cargo.toml" → Blob
//! ```
//!
//! Each entry has:
//! - A **name** (file or directory name, e.g., `"README.md"`).
//! - A **kind** (`EntryKind::Blob` for files, `EntryKind::Tree` for subdirs).
//! - A **hash** that points to the content (a `Blob` or another `Tree`).
//!
//! ## Invariants
//!
//! To ensure deterministic hashing and efficient binary search, a `Tree` enforces:
//!
//! 1. **Sorted order** – entries are sorted lexicographically by name.
//! 2. **No duplicates** – two entries cannot have the same name.
//!
//! These invariants are checked at construction time. If violated,
//! [`Tree::new`] returns [`VctrlError::InvalidName`].
//!
//! ## Why Sorted and Unique?
//!
//! - **Deterministic hashing** – the same directory content always produces
//!   the same hash, regardless of insertion order.
//! - **Efficient lookup** – binary search can be used to find entries by name.
//! - **Consistency** – the representation is canonical, making it easier to
//!   compare two trees for equality.
//!
//! ## Example
//!
//! ```rust
//! use libvctrl_handler::{Hash, Tree, TreeEntry, EntryKind, HASH_LENGTH};
//!
//! # let hash = Hash::from_bytes(&[0xAA; HASH_LENGTH]).unwrap();
//! let readme = TreeEntry::new("README.md".into(), EntryKind::Blob, hash).unwrap();
//! let src_dir = TreeEntry::new("src".into(), EntryKind::Tree, hash).unwrap();
//!
//! // Entries must be sorted lexicographically: "README.md" < "src"
//! let tree = Tree::new(vec![readme, src_dir]).unwrap();
//! assert_eq!(tree.entries().len(), 2);
//! ```
//!
//! ## Relation to Other Types
//!
//! - [`Commit`](crate::Commit) – points to a root tree (the top‑level directory).
//! - [`TreeEntry`] – the building block of a tree.
//! - [`Blob`](crate::Blob) – the content of a file, referenced by a `TreeEntry`.
//! - [`Hash`](crate::Hash) – the identifier of a tree, computed from its encoded bytes.

use crate::enums::EntryKind;
use crate::errors::VctrlError;
use crate::types::hash::Hash;

use super::validate_name;

/// A single entry inside a [`Tree`].
///
/// An entry is the basic building block of a directory listing. It pairs
/// a **name** with a **kind** (blob or subtree) and a **hash** that points to
/// the actual content.
///
/// # Validation
/// The name must be non‑empty and ≤ [`MAX_NAME_LENGTH`](crate::constants::MAX_NAME_LENGTH).
///
/// # Example
///
/// ```rust
/// use libvctrl_handler::{Hash, TreeEntry, EntryKind, HASH_LENGTH};
///
/// # let hash = Hash::from_bytes(&[0x11; HASH_LENGTH]).unwrap();
/// // A file entry.
/// let file = TreeEntry::new("src/main.rs".into(), EntryKind::Blob, hash)
///     .expect("valid entry");
/// assert_eq!(file.name(), "src/main.rs");
/// assert_eq!(file.kind(), EntryKind::Blob);
/// assert_eq!(file.hash().as_bytes().len(), HASH_LENGTH);
///
/// // An empty name is rejected.
/// assert!(TreeEntry::new("".into(), EntryKind::Blob, hash).is_err());
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeEntry {
    name: String,
    kind: EntryKind,
    hash: Hash,
}

impl TreeEntry {
    /// Creates a new `TreeEntry` after validating the name.
    ///
    /// # Errors
    /// Returns [`VctrlError::InvalidName`] if `name` is empty or too long.
    pub fn new(name: String, kind: EntryKind, hash: Hash) -> Result<Self, VctrlError> {
        validate_name(&name)?;
        Ok(Self { name, kind, hash })
    }

    /// Returns the name of the entry.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the kind of the entry (blob or tree).
    #[must_use]
    pub const fn kind(&self) -> EntryKind {
        self.kind
    }

    /// Returns the hash of the entry (points to a [`Blob`](crate::Blob) or another [`Tree`]).
    #[must_use]
    pub const fn hash(&self) -> &Hash {
        &self.hash
    }
}

/// A tree object – a virtual directory listing.
///
/// A tree contains a **sorted** list of [`TreeEntry`] items. Entries are
/// ordered lexicographically by name, and duplicate names are forbidden.
/// These invariants are enforced at construction time.
///
/// # Errors
/// [`Tree::new`] will return an error if:
/// - Entries are not in sorted order.
/// - Two entries share the same name.
///
/// # Example
///
/// ```rust
/// use libvctrl_handler::{Hash, Tree, TreeEntry, EntryKind, HASH_LENGTH};
///
/// # let hash = Hash::from_bytes(&[0x22; HASH_LENGTH]).unwrap();
/// // Create sorted entries.
/// let file = TreeEntry::new("a.txt".into(), EntryKind::Blob, hash).unwrap();
/// let dir  = TreeEntry::new("sub".into(), EntryKind::Tree, hash).unwrap();
///
/// // Build the tree – entries must be in order.
/// let tree = Tree::new(vec![file, dir]).expect("sorted entries");
/// assert_eq!(tree.entries().len(), 2);
///
/// // Duplicate names are rejected.
/// let dup1 = TreeEntry::new("x".into(), EntryKind::Blob, hash).unwrap();
/// let dup2 = TreeEntry::new("x".into(), EntryKind::Blob, hash).unwrap();
/// assert!(Tree::new(vec![dup1, dup2]).is_err());
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tree {
    entries: Vec<TreeEntry>,
}

impl Tree {
    /// Creates a new `Tree` from a vector of entries.
    ///
    /// The entries must be **sorted in ascending lexicographic order** by name
    /// and must contain **no duplicate names**. This ensures a canonical
    /// representation for hashing and efficient operations.
    ///
    /// # Errors
    /// Returns [`VctrlError::InvalidName`] with a descriptive message if:
    /// - Any adjacent pair is out of order (`entries[i-1].name() >= entries[i].name()`).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use libvctrl_handler::*;
    /// # let hash = Hash::from_bytes(&[0x33; 64]).unwrap();
    /// let entries = vec![
    ///     TreeEntry::new("bar".into(), EntryKind::Blob, hash).unwrap(),
    ///     TreeEntry::new("foo".into(), EntryKind::Blob, hash).unwrap(),
    /// ];
    /// let tree = Tree::new(entries).unwrap();
    /// ```
    ///
    /// # Panics
    /// This method does not panic, but it returns a `Result`; the caller must
    /// handle the error appropriately.
    pub fn new(entries: Vec<TreeEntry>) -> Result<Self, VctrlError> {
        for i in 1..entries.len() {
            if entries[i - 1].name() >= entries[i].name() {
                return Err(VctrlError::InvalidName(format!(
                    "Tree entries are not sorted or contain duplicates: '{}' vs '{}'",
                    entries[i - 1].name(),
                    entries[i].name()
                )));
            }
        }
        Ok(Self { entries })
    }

    /// Returns a reference to the list of entries.
    ///
    /// The returned slice is guaranteed to be sorted and contain no duplicates.
    #[must_use]
    pub fn entries(&self) -> &[TreeEntry] {
        &self.entries
    }
}
