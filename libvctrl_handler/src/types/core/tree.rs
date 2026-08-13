//! Tree objects for representing directory snapshots.
//!
//! # Purpose
//!
//! A [`Tree`] stores a sorted list of [`TreeEntry`] items, each associating
//! a file or directory name with its [`Hash`] and the kind of object it
//! points to (blob or subtree). Trees are the backbone of the
//! content-addressable filesystem model: they encode the structure of a
//! directory at a specific point in time.
//!
//! # Design Constraints
//!
//! - **Sorted entries**: Entries must be strictly increasing in name order
//!   (no duplicates, no equal names). This deterministic ordering ensures
//!   that the same directory contents always produce the identical tree
//!   object and thus the same hash.
//! - **Name validation**: Each entry name is validated via
//!   [`validate_tree_entry_name`](crate::types::validate_tree_entry_name),
//!   which forbids `/`, `.`, and `..`. This enforces flat, simple names and
//!   prevents path-traversal bugs.
//! - **Immutability**: Once a tree is created, its entries cannot be changed.
//!   Mutable operations create a new tree.
//!
//! # Relationship to Other Types
//!
//! A [`Tree`] is a node in the repository object graph. Each [`TreeEntry`]
//! points to either a [`Blob`](crate::Blob) (representing file content) or
//! another [`Tree`] (representing a subdirectory). The [`Hash`] stored in
//! each entry is the content address of the referenced object. This forms a
//! Merkle DAG, where the tree's own hash is derived from its entries, which
//! in turn reference child objects.
//!
//! # Memory Layout
//!
//! A [`Tree`] owns a [`Vec<TreeEntry>`], which is a heap-allocated buffer
//! containing the entries. Each [`TreeEntry`] owns a [`String`] for the name,
//! stores an [`EntryKind`] discriminant (one byte), and a [`Hash`] (64 bytes).
//! The tree is not `Copy` because it owns heap-allocated data; cloning
//! performs a deep copy of the entry list and all names.
//!
//! # Why Sorted?
//!
//! Deterministic ordering is crucial for content addressing. If two trees
//! have the same set of entries but in different orders, they would produce
//! different hashes. By enforcing a canonical lexicographic order at
//! construction, we guarantee that identical directory contents hash
//! identically regardless of the order in which entries were inserted.
//!
//! # Examples
//!
//! Building a tree with two entries:
//!
//! ```
//! use libvctrl_handler::types::core::{Tree, TreeEntry, Hash};
//! use libvctrl_handler::enums::EntryKind;
//! use libvctrl_handler::constants::HASH_LENGTH;
//!
//! let blob_hash = Hash::from_bytes(&[0x11; HASH_LENGTH]).unwrap();
//! let tree_hash = Hash::from_bytes(&[0x22; HASH_LENGTH]).unwrap();
//!
//! let entries = vec![
//!     TreeEntry::new("file.txt".into(), EntryKind::Blob, blob_hash).unwrap(),
//!     TreeEntry::new("subdir".into(), EntryKind::Tree, tree_hash).unwrap(),
//! ];
//!
//! let tree = Tree::new(entries).unwrap();
//! assert_eq!(tree.entries().len(), 2);
//! ```

use super::hash::Hash;
use crate::enums::EntryKind;
use crate::errors::VctrlError;
use crate::types::validate_tree_entry_name;

/// A single entry in a [`Tree`], representing a file or subdirectory.
///
/// # Purpose
///
/// Each entry binds a **name**, a **kind** (blob or tree), and the **hash**
/// of the referenced object. A [`TreeEntry`] is the fundamental link between
/// a pathname in a directory and the content-addressable object that backs
/// it.
///
/// # Design Rationale
///
/// The fields are private to guarantee that once constructed with a valid
/// name, no code can accidentally change the name, kind, or hash. This
/// preserves the tree's integrity and hash. Accessor methods provide
/// read-only access to each field.
///
/// ## Why not public fields?
///
/// If fields were public, a caller could mutate an entry after it was placed
/// in a [`Tree`], breaking the tree's canonical ordering or changing the
/// referenced object. Private fields with a validated constructor ensure
/// that an entry is always well-formed.
///
/// ## Relationship to Unix mode bits
///
/// The [`EntryKind`] stored in this struct is the logical object kind. The
/// raw Unix mode bits used during serialization are defined in
/// [`crate::constants::entry_mode`]. Decoder and encoder implementations
/// translate between the two representations.
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
/// let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
/// let entry = TreeEntry::new("README.md".into(), EntryKind::Blob, hash).unwrap();
/// assert_eq!(entry.name(), "README.md");
/// assert_eq!(entry.kind(), EntryKind::Blob);
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
    /// The `name` must be non-empty, not exceed the maximum length, and
    /// must not contain `/`, `.`, or `..` as a component.
    ///
    /// # Arguments
    ///
    /// * `name` - The entry name (e.g., `"README.md"`). It is moved into
    ///   the entry.
    /// * `kind` - The [`EntryKind`] of the object this entry points to.
    /// * `hash` - The [`Hash`] of the referenced object.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidName`] if the name fails validation.
    ///
    /// # Why Fallible?
    ///
    /// Tree entry names are security-sensitive. They must be flat and simple
    /// to prevent path traversal and to ensure correct serialization. The
    /// constructor returns a [`Result`] to force callers to handle invalid
    /// input immediately.
    ///
    /// # How It Works Internally
    ///
    /// 1. Calls [`validate_tree_entry_name`] on the provided name.
    /// 2. If validation fails, returns an error.
    /// 3. Otherwise, constructs the entry with the validated name, kind,
    ///    and hash, and wraps it in `Ok`.
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
    /// let hash = Hash::from_bytes(&[0xaa; HASH_LENGTH]).unwrap();
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
    /// let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    /// let err = TreeEntry::new("a/b".into(), EntryKind::Blob, hash).unwrap_err();
    /// assert!(matches!(err, libvctrl_handler::VctrlError::InvalidName(_)));
    /// ```
    pub fn new(name: String, kind: EntryKind, hash: Hash) -> Result<Self, VctrlError> {
        validate_tree_entry_name(&name)?;
        Ok(Self { name, kind, hash })
    }

    /// Returns the entry name.
    ///
    /// # Returns
    ///
    /// A string slice containing the validated entry name.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{TreeEntry, Hash};
    /// use libvctrl_handler::enums::EntryKind;
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    /// let entry = TreeEntry::new("Cargo.toml".into(), EntryKind::Blob, hash).unwrap();
    /// assert_eq!(entry.name(), "Cargo.toml");
    /// ```
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the [`EntryKind`] of this entry (blob, executable, symlink,
    /// tree, or submodule).
    ///
    /// # Returns
    ///
    /// The logical object kind of the referenced object. This is a [`Copy`]
    /// type, so the returned value is independent of the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{TreeEntry, Hash};
    /// use libvctrl_handler::enums::EntryKind;
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    /// let entry = TreeEntry::new("script.sh".into(), EntryKind::Executable, hash).unwrap();
    /// assert_eq!(entry.kind(), EntryKind::Executable);
    /// ```
    #[must_use]
    pub const fn kind(&self) -> EntryKind {
        self.kind
    }

    /// Returns the hash of the object this entry points to.
    ///
    /// # Returns
    ///
    /// A reference to the [`Hash`] of the referenced object. The reference
    /// is borrowed from the entry, so it lives as long as the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{TreeEntry, Hash};
    /// use libvctrl_handler::enums::EntryKind;
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// let hash = Hash::from_bytes(&[0x5A; HASH_LENGTH]).unwrap();
    /// let entry = TreeEntry::new("file.bin".into(), EntryKind::Blob, hash).unwrap();
    /// assert_eq!(entry.hash(), &hash);
    /// ```
    #[must_use]
    pub const fn hash(&self) -> &Hash {
        &self.hash
    }
}

/// A sorted list of [`TreeEntry`] representing a directory snapshot.
///
/// # Purpose
///
/// A `Tree` is the version-control equivalent of a directory. It contains
/// zero or more entries, each referencing a file ([`Blob`](crate::Blob)) or
/// subdirectory (another `Tree`). The entries are kept in lexicographic
/// order by name, enforced at construction time.
///
/// # Design Rationale
///
/// - **Immutability**: The entries vector is private and cannot be mutated
///   after construction. This ensures the tree's hash remains stable.
/// - **Sorted order**: Strict sorting by name is enforced in
///   [`Tree::new`]. This is essential for deterministic hashing and
///   efficient binary search during lookups.
/// - **Validation**: Each entry is individually validated by
///   [`TreeEntry::new`], and the vector as a whole is validated for ordering
///   in [`Tree::new`].
///
/// # Why Sorted?
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
/// let blob_hash = Hash::from_bytes(&[0x11; HASH_LENGTH]).unwrap();
/// let tree_hash = Hash::from_bytes(&[0x22; HASH_LENGTH]).unwrap();
///
/// let entries = vec![
///     TreeEntry::new("file.txt".into(), EntryKind::Blob, blob_hash).unwrap(),
///     TreeEntry::new("subdir".into(), EntryKind::Tree, tree_hash).unwrap(),
/// ];
///
/// let tree = Tree::new(entries).unwrap();
/// assert_eq!(tree.entries().len(), 2);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tree {
    entries: Vec<TreeEntry>,
}

impl Tree {
    /// Creates a new `Tree` from a vector of entries, validating the sort
    /// order.
    ///
    /// The provided `entries` must be strictly sorted by name (no duplicates,
    /// no equal names). This is checked by iterating over adjacent pairs.
    ///
    /// # Arguments
    ///
    /// * `entries` - A vector of [`TreeEntry`] items in strict ascending
    ///   lexicographic order by name.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidName`] if any two adjacent entries are
    /// not in strict ascending lexicographic order (i.e., the latter is not
    /// strictly greater than the former).
    ///
    /// # Why Validate Sort Order?
    ///
    /// The sort order is a core invariant of a tree. It ensures that two
    /// trees with the same set of entries but different insertion orders
    /// produce the same hash. Without this invariant, content addressing
    /// would be non-deterministic.
    ///
    /// # How It Works Internally
    ///
    /// 1. The method iterates from index 1 to `entries.len() - 1`.
    /// 2. For each adjacent pair, it compares the names using the `>=`
    ///    operator on `&str`.
    /// 3. If an out-of-order or duplicate pair is found, it returns an
    ///    [`VctrlError::InvalidName`] with a descriptive message.
    /// 4. If all pairs are strictly increasing, it constructs the tree and
    ///    returns `Ok`.
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
    /// let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    /// let entries = vec![
    ///     TreeEntry::new("a".into(), EntryKind::Blob, hash).unwrap(),
    ///     TreeEntry::new("b".into(), EntryKind::Blob, hash).unwrap(),
    /// ];
    /// let tree = Tree::new(entries).unwrap();
    /// assert_eq!(tree.entries().len(), 2);
    /// ```
    ///
    /// Duplicate names cause failure:
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Tree, TreeEntry, Hash};
    /// use libvctrl_handler::enums::EntryKind;
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    /// let entries = vec![
    ///     TreeEntry::new("file".into(), EntryKind::Blob, hash).unwrap(),
    ///     TreeEntry::new("file".into(), EntryKind::Blob, hash).unwrap(),
    /// ];
    /// let err = Tree::new(entries).unwrap_err();
    /// assert!(matches!(err, libvctrl_handler::VctrlError::InvalidName(_)));
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
    /// # Returns
    ///
    /// A slice of all [`TreeEntry`] items in the tree, in sorted order. The
    /// slice borrows from the tree; no copying occurs.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Tree, TreeEntry, Hash};
    /// use libvctrl_handler::enums::EntryKind;
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    /// let entries = vec![
    ///     TreeEntry::new("Cargo.toml".into(), EntryKind::Blob, hash).unwrap(),
    /// ];
    /// let tree = Tree::new(entries).unwrap();
    ///
    /// assert_eq!(tree.entries().len(), 1);
    /// assert_eq!(tree.entries()[0].name(), "Cargo.toml");
    /// ```
    #[must_use]
    pub fn entries(&self) -> &[TreeEntry] {
        &self.entries
    }
}
