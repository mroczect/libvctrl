//! Tree objects for representing directory snapshots.
//!
//! A [`Tree`] stores a sorted list of [`TreeEntry`] items, each associating
//! a file or directory name with its [`Hash`] and the kind of object it
//! points to (blob or subtree). Trees are the backbone of the content‑addressable
//! filesystem model: they encode the structure of a directory at a specific
//! point in time.
//!
//! ## Design Constraints
//!
//! - **Sorted entries**: Entries must be strictly increasing in name order
//!   (no duplicates, no equal names). This deterministic ordering ensures
//!   that the same directory contents always produce the identical tree
//!   object and thus the same hash.
//! - **Name validation**: Each entry name is validated via
//!   [`validate_tree_entry_name`](crate::types::validate_tree_entry_name),
//!   which forbids `/`, `.`, and `..`. This enforces flat, simple names and
//!   prevents path‑traversal bugs.
//! - **Immutability**: Once a tree is created, its entries cannot be changed.
//!   Mutable operations create a new tree.

use super::hash::Hash;
use crate::enums::EntryKind;
use crate::errors::VctrlError;
use crate::types::validate_tree_entry_name;

/// A single entry in a [`Tree`], representing a file or subdirectory.
///
/// Each entry binds a **name**, a **kind** (blob or tree), and the **hash**
/// of the referenced object. Names are subject to the same validation rules
/// as file‑names in a tree: no directory separators, and not `.` or `..`.
///
/// # Why private fields?
///
/// The fields are private to guarantee that once constructed with a valid
/// name, no code can accidentally change the name, kind, or hash. This
/// preserves the tree's integrity and hash.
///
/// # Examples
///
/// Creating a valid entry:
///
/// ```
/// use libvctrl_handler::types::core::{TreeEntry, Hash};
/// use libvctrl_handler::enums::EntryKind;
/// use libvctrl_handler::constants::HASH_LENGTH;
///
/// # let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
/// let entry = TreeEntry::new("README.md".into(), EntryKind::Blob, hash).unwrap();
/// assert_eq!(entry.name(), "README.md");
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
    /// The `name` must be non‑empty, not exceed the maximum length, and
    /// must not contain `/`, `.`, or `..` as a component.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidName`] if the name fails validation.
    ///
    /// # Examples
    ///
    /// Successful creation:
    ///
    /// ```
    /// use libvctrl_handler::types::core::{TreeEntry, Hash};
    /// use libvctrl_handler::enums::EntryKind;
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// # let hash = Hash::from_bytes(&[0xaa; HASH_LENGTH]).unwrap();
    /// let entry = TreeEntry::new("src".into(), EntryKind::Tree, hash).unwrap();
    /// assert_eq!(entry.kind(), EntryKind::Tree);
    /// ```
    ///
    /// Invalid name:
    ///
    /// ```
    /// use libvctrl_handler::types::core::{TreeEntry, Hash};
    /// use libvctrl_handler::enums::EntryKind;
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// # let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    /// let err = TreeEntry::new("a/b".into(), EntryKind::Blob, hash).unwrap_err();
    /// assert!(matches!(err, libvctrl_handler::errors::VctrlError::InvalidName(_)));
    /// ```
    pub fn new(name: String, kind: EntryKind, hash: Hash) -> Result<Self, VctrlError> {
        validate_tree_entry_name(&name)?;
        Ok(Self { name, kind, hash })
    }

    /// Returns the entry name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the [`EntryKind`] of this entry (blob or tree).
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

/// A sorted list of [`TreeEntry`] representing a directory snapshot.
///
/// A `Tree` is the version‑control equivalent of a directory. It contains
/// zero or more entries, each referencing a file ([`Blob`]) or subdirectory
/// (another `Tree`). The entries are kept in lexicographic order by name,
/// enforced at construction time.
///
/// # Why sorted?
///
/// Deterministic ordering is crucial for content addressing. If two trees
/// have the same set of entries but in different orders, they would produce
/// different hashes. By enforcing a canonical order, we guarantee that
/// identical directory contents hash identically.
///
/// # Examples
///
/// Building a tree with two entries:
///
/// ```
/// use libvctrl_handler::types::core::{Tree, TreeEntry, Hash};
/// use libvctrl_handler::enums::EntryKind;
/// use libvctrl_handler::constants::HASH_LENGTH;
///
/// # let blob_hash = Hash::from_bytes(&[0x11; HASH_LENGTH]).unwrap();
/// # let tree_hash = Hash::from_bytes(&[0x22; HASH_LENGTH]).unwrap();
/// let entries = vec![
///     TreeEntry::new("file.txt".into(), EntryKind::Blob, blob_hash).unwrap(),
///     TreeEntry::new("subdir".into(), EntryKind::Tree, tree_hash).unwrap(),
/// ];
/// let tree = Tree::new(entries).unwrap();
/// assert_eq!(tree.entries().len(), 2);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tree {
    entries: Vec<TreeEntry>,
}

impl Tree {
    /// Creates a new `Tree` from a vector of entries, validating the sort order.
    ///
    /// The provided `entries` must be strictly sorted by name (no duplicates,
    /// no equal names). This is checked by iterating over adjacent pairs.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidName`] if any two adjacent entries are not in
    /// strict ascending lexicographic order (i.e., the latter is not strictly
    /// greater than the former).
    ///
    /// # Examples
    ///
    /// Correctly sorted entries:
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Tree, TreeEntry, Hash};
    /// use libvctrl_handler::enums::EntryKind;
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// # let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    /// let entries = vec![
    ///     TreeEntry::new("a".into(), EntryKind::Blob, hash).unwrap(),
    ///     TreeEntry::new("b".into(), EntryKind::Blob, hash).unwrap(),
    /// ];
    /// let tree = Tree::new(entries).unwrap();
    /// ```
    ///
    /// Duplicate names cause failure:
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Tree, TreeEntry, Hash};
    /// use libvctrl_handler::enums::EntryKind;
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// # let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    /// let entries = vec![
    ///     TreeEntry::new("file".into(), EntryKind::Blob, hash).unwrap(),
    ///     TreeEntry::new("file".into(), EntryKind::Blob, hash).unwrap(),
    /// ];
    /// let err = Tree::new(entries).unwrap_err();
    /// assert!(matches!(err, libvctrl_handler::errors::VctrlError::InvalidName(_)));
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

    /// Returns the entries of the tree as a slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Tree, TreeEntry, Hash};
    /// use libvctrl_handler::enums::EntryKind;
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// # let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    /// let entries = vec![
    ///     TreeEntry::new("Cargo.toml".into(), EntryKind::Blob, hash).unwrap(),
    /// ];
    /// let tree = Tree::new(entries).unwrap();
    /// assert_eq!(tree.entries()[0].name(), "Cargo.toml");
    /// ```
    #[must_use]
    pub fn entries(&self) -> &[TreeEntry] {
        &self.entries
    }
}
