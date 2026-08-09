//! Directory tree representation for version control systems.
//!
//! A [`Tree`] is an ordered collection of [`TreeEntry`] items, each mapping a
//! file or sub‑directory name to its hash and kind. Trees are the backbone of
//! the repository object model, connecting blobs and subtrees into a single
//! snapshot.

use crate::enums::EntryKind;
use crate::errors::VctrlError;
use crate::types::hash::Hash;

use super::validate_name;

/// A single entry in a directory tree.
///
/// Each entry associates a `name` with a `hash` of the object it points to
/// and a `kind` indicating whether the object is a blob, tree, or other.
/// Entries are immutable once constructed.
///
/// # Design
///
/// Fields are private to enforce consistency: the name is validated at
/// construction time, and the hash and kind cannot be changed afterwards.
/// The struct is [`Clone`], [`Debug`], [`PartialEq`], and [`Eq`] so that
/// trees can be compared and duplicated efficiently.
///
/// # Examples
///
/// Creating a blob entry:
///
/// ```
/// # use libvctrl_handler::{EntryKind, Hash, TreeEntry};
/// # fn make_hash() -> Hash {
/// #     let bytes = [0xABu8; 64];
/// #     Hash::from_bytes(&bytes).unwrap()
/// # }
/// let hash = make_hash();
/// let entry = TreeEntry::new("README.md".into(), EntryKind::Blob, hash).unwrap();
///
/// assert_eq!(entry.name(), "README.md");
/// assert_eq!(entry.kind(), EntryKind::Blob);
/// ```
///
/// Attempting to create an entry with an empty name fails:
///
/// ```
/// # use libvctrl_handler::{EntryKind, Hash, TreeEntry};
/// # fn make_hash() -> Hash {
/// #     let bytes = [0x00u8; 64];
/// #     Hash::from_bytes(&bytes).unwrap()
/// # }
/// let hash = make_hash();
/// assert!(TreeEntry::new("".into(), EntryKind::Blob, hash).is_err());
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeEntry {
    name: String,
    kind: EntryKind,
    hash: Hash,
}

impl TreeEntry {
    /// Creates a new [`TreeEntry`] with the given name, kind, and hash.
    ///
    /// The `name` is validated via the internal `validate_name` helper. It
    /// must be non‑empty and not exceed the maximum name length.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidName`] if the name fails validation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{EntryKind, Hash, TreeEntry};
    /// # fn make_hash() -> Hash {
    /// #     let bytes = [0x11u8; 64];
    /// #     Hash::from_bytes(&bytes).unwrap()
    /// # }
    /// let hash = make_hash();
    /// let entry = TreeEntry::new("src/main.rs".into(), EntryKind::Blob, hash).unwrap();
    /// assert_eq!(entry.name(), "src/main.rs");
    /// ```
    pub fn new(name: String, kind: EntryKind, hash: Hash) -> Result<Self, VctrlError> {
        validate_name(&name)?;
        Ok(Self { name, kind, hash })
    }

    /// Returns the entry’s file or directory name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the kind of object this entry points to.
    #[must_use]
    pub const fn kind(&self) -> EntryKind {
        self.kind
    }

    /// Returns the hash of the object this entry points to.
    #[must_use]
    pub const fn hash(&self) -> &Hash {
        &self.hash
    }
}

/// An ordered collection of directory entries.
///
/// A `Tree` represents the contents of a single directory in the repository.
/// Entries are stored in a sorted order and duplicates are not allowed. The
/// sorting must be in ascending byte‑lexicographic order of the entry
/// names, as is conventional in many version control systems. This ordering
/// ensures deterministic tree hashes.
///
/// # Design
///
/// The entries are owned by the tree and cannot be modified after
/// construction. The constructor [`Tree::new`] validates that the provided
/// entries are strictly increasing in name order and that no name exceeds
/// the maximum length. If validation fails, an [`VctrlError::InvalidName`]
/// is returned.
///
/// # Examples
///
/// Building a simple tree with two entries:
///
/// ```
/// # use libvctrl_handler::{EntryKind, Hash, Tree, TreeEntry};
/// # fn make_hash() -> Hash {
/// #     let bytes = [0xCCu8; 64];
/// #     Hash::from_bytes(&bytes).unwrap()
/// # }
/// let hash = make_hash();
/// let entry1 = TreeEntry::new("file.txt".into(), EntryKind::Blob, hash).unwrap();
/// let entry2 = TreeEntry::new("subdir".into(), EntryKind::Tree, hash).unwrap();
/// let tree = Tree::new(vec![entry1, entry2]).unwrap();
///
/// assert_eq!(tree.entries().len(), 2);
/// assert_eq!(tree.entries()[0].name(), "file.txt");
/// assert_eq!(tree.entries()[1].name(), "subdir");
/// ```
///
/// Attempting to create a tree with unsorted or duplicate entries fails:
///
/// ```
/// # use libvctrl_handler::{EntryKind, Hash, Tree, TreeEntry};
/// # fn make_hash() -> Hash {
/// #     let bytes = [0xDDu8; 64];
/// #     Hash::from_bytes(&bytes).unwrap()
/// # }
/// let hash = make_hash();
/// let entry1 = TreeEntry::new("b".into(), EntryKind::Blob, hash).unwrap();
/// let entry2 = TreeEntry::new("a".into(), EntryKind::Blob, hash).unwrap();
/// assert!(Tree::new(vec![entry1, entry2]).is_err()); // "b" > "a"
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tree {
    entries: Vec<TreeEntry>,
}

impl Tree {
    /// Creates a new [`Tree`] from a vector of entries.
    ///
    /// The provided `entries` must be sorted in ascending order by their
    /// names and contain no duplicates. An empty vector is allowed and
    /// represents an empty directory.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidName`] if any two consecutive entries
    /// are not strictly ordered (i.e., if an entry name is ≥ the name of
    /// the next entry).
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{EntryKind, Hash, Tree, TreeEntry};
    /// # fn make_hash() -> Hash {
    /// #     let bytes = [0xEEu8; 64];
    /// #     Hash::from_bytes(&bytes).unwrap()
    /// # }
    /// let hash = make_hash();
    /// let e1 = TreeEntry::new("a".into(), EntryKind::Blob, hash).unwrap();
    /// let e2 = TreeEntry::new("b".into(), EntryKind::Blob, hash).unwrap();
    /// let tree = Tree::new(vec![e1, e2]).unwrap();
    /// assert_eq!(tree.entries().len(), 2);
    /// ```
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

    /// Returns a slice of all entries in the tree, in sorted order.
    #[must_use]
    pub fn entries(&self) -> &[TreeEntry] {
        &self.entries
    }
}
