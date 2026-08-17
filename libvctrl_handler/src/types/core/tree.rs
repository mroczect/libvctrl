//! Tree object and entry representation.
//!
//! # Architecture
//! This module defines the [`Tree`] and [`TreeEntry`] structs, which represent
//! directory listings in the Git object model. A tree maps names to modes and
//! object hashes, forming the hierarchical structure of a repository snapshot.
//!
//! # Design Rationale: Canonical Sorting
//! Git requires tree entries to be sorted in a very specific, canonical order to
//! ensure that identical directory states always produce identical hashes. This
//! module enforces that sorting rule via the private `compare_tree_entries`
//! function. By sorting upon construction, the [`Tree::new`] method guarantees
//! that any `Tree` instance in memory is immediately valid and ready for hashing.

use super::hash::Hash;
use crate::constants::MAX_TREE_ENTRIES;
use crate::enums::EntryKind;
use crate::errors::VctrlError;
use crate::validation::validate_tree_entry_name;
use std::cmp::Ordering;

/// A single entry in a Git tree.
///
/// # Why this exists
/// Represents the atomic mapping between a filename, its filesystem mode
/// ([`EntryKind`]), and its content hash ([`Hash`]). By requiring construction
/// via [`new`](Self::new), the crate ensures that every entry name is validated,
/// preventing path traversal vulnerabilities (e.g., names containing `/` or `..`).
///
/// # Examples
///
/// Creating a valid tree entry:
///
/// ```
/// # use libvctrl_handler::types::core::tree::TreeEntry;
/// # use libvctrl_handler::types::core::hash::Hash;
/// # use libvctrl_handler::enums::EntryKind;
/// # use libvctrl_handler::VctrlError;
/// # let hash = Hash::from_bytes(&[0_u8; 64])?;
/// let entry = TreeEntry::new("main.rs".to_string(), EntryKind::Blob, hash)?;
/// assert_eq!(entry.name(), "main.rs");
/// # Ok::<(), VctrlError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeEntry {
    name: String,
    kind: EntryKind,
    hash: Hash,
}

impl TreeEntry {
    /// Creates a new tree entry.
    ///
    /// # How it works
    /// Delegates to [`validate_tree_entry_name`](crate::validation::validate_tree_entry_name)
    /// to ensure the name is a single path component without forbidden characters.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidName`] if the entry name is invalid (e.g., contains slashes).
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::types::core::tree::TreeEntry;
    /// # use libvctrl_handler::types::core::hash::Hash;
    /// # use libvctrl_handler::enums::EntryKind;
    /// # use libvctrl_handler::VctrlError;
    /// # let hash = Hash::from_bytes(&[0_u8; 64])?;
    /// assert!(TreeEntry::new("valid.txt".into(), EntryKind::Blob, hash).is_ok());
    /// assert!(TreeEntry::new("invalid/path.txt".into(), EntryKind::Blob, hash).is_err());
    /// # Ok::<(), VctrlError>(())
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

    /// Returns the entry kind.
    #[must_use]
    pub const fn kind(&self) -> EntryKind {
        self.kind
    }

    /// Returns the hash of the entry.
    #[must_use]
    pub const fn hash(&self) -> &Hash {
        &self.hash
    }
}

/// A Git tree object (directory listing).
///
/// Entries are always stored in Git-sorted order: tree entries (directories)
/// are compared as if their name has a trailing `/` appended.
///
/// # Why this exists
/// Provides a strongly-typed, validated representation of a directory. By sorting
/// and checking for duplicates upon construction, the [`Tree::new`] method acts as
/// a gatekeeper, guaranteeing that any `Tree` instance in memory is structurally
/// sound and ready to be serialized into a canonical format.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tree {
    entries: Vec<TreeEntry>,
}

impl Tree {
    /// Creates a new tree from a vector of entries.
    ///
    /// Entries are sorted according to Git tree ordering rules.
    /// Duplicate entry names are rejected.
    ///
    /// # How it works
    /// 1. Checks the entry count against [`MAX_TREE_ENTRIES`](crate::constants::MAX_TREE_ENTRIES).
    /// 2. Sorts the entries in-place using `compare_tree_entries`.
    /// 3. Scans for duplicate names using a sliding window (`windows(2)`), rejecting
    ///    the tree if any are found.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::ExceededMaxSize`] if the entry count exceeds `MAX_TREE_ENTRIES`.
    /// Returns [`VctrlError::InvalidTreeStructure`] if duplicate names are found.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::types::core::tree::{Tree, TreeEntry};
    /// # use libvctrl_handler::types::core::hash::Hash;
    /// # use libvctrl_handler::enums::EntryKind;
    /// # use libvctrl_handler::VctrlError;
    /// # let hash = Hash::from_bytes(&[0_u8; 64])?;
    /// let e1 = TreeEntry::new("b.txt".into(), EntryKind::Blob, hash)?;
    /// let e2 = TreeEntry::new("a.txt".into(), EntryKind::Blob, hash)?;
    /// let tree = Tree::new(vec![e1, e2])?;
    /// // Entries are sorted automatically
    /// assert_eq!(tree.entries()[0].name(), "a.txt");
    /// # Ok::<(), VctrlError>(())
    /// ```
    pub fn new(entries: Vec<TreeEntry>) -> Result<Self, VctrlError> {
        let max_entries = usize::try_from(MAX_TREE_ENTRIES).unwrap_or(usize::MAX);
        if entries.len() > max_entries {
            return Err(VctrlError::ExceededMaxSize(format!(
                "tree has {} entries, exceeding maximum of {MAX_TREE_ENTRIES}",
                entries.len()
            )));
        }

        let mut sorted = entries;
        sorted.sort_by(compare_tree_entries);

        for window in sorted.windows(2) {
            if let (Some(first), Some(second)) = (window.first(), window.get(1))
                && first.name == second.name
            {
                return Err(VctrlError::InvalidTreeStructure(format!(
                    "duplicate entry name: '{}'",
                    first.name
                )));
            }
        }

        Ok(Self { entries: sorted })
    }

    /// Returns the tree entries in Git-sorted order.
    #[must_use]
    pub fn entries(&self) -> &[TreeEntry] {
        &self.entries
    }

    /// Returns the number of entries.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the tree has no entries.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Looks up an entry by name.
    ///
    /// # How it works
    /// Performs a linear scan. While binary search is possible due to the sorted
    /// nature of the entries, linear scan is often faster for small vectors typical
    /// of Git trees due to CPU cache locality.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&TreeEntry> {
        self.entries.iter().find(|e| e.name == name)
    }
}

/// Compares two tree entries using Git ordering rules.
///
/// Tree entries (directories) are compared as if their name has a
/// trailing `/` appended. All other kinds use their name as-is.
///
/// # How it works
/// The function compares byte-by-byte. If one name is a prefix of the other,
/// the shorter name is padded with a virtual `/` if it represents a tree.
/// This ensures that `a` (blob) sorts before `a` (tree), which sorts before `ab` (blob).
#[inline]
fn compare_tree_entries(a: &TreeEntry, b: &TreeEntry) -> Ordering {
    let a_bytes = a.name.as_bytes();
    let b_bytes = b.name.as_bytes();
    let a_is_tree = a.kind == EntryKind::Tree;
    let b_is_tree = b.kind == EntryKind::Tree;

    let a_len = a_bytes.len() + usize::from(a_is_tree);
    let b_len = b_bytes.len() + usize::from(b_is_tree);
    let min_len = a_len.min(b_len);

    for i in 0..min_len {
        let a_byte = a_bytes.get(i).copied().unwrap_or(b'/');
        let b_byte = b_bytes.get(i).copied().unwrap_or(b'/');
        match a_byte.cmp(&b_byte) {
            Ordering::Equal => {}
            Ordering::Less => return Ordering::Less,
            Ordering::Greater => return Ordering::Greater,
        }
    }

    a_len.cmp(&b_len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::HASH_LENGTH;

    fn zero_hash() -> Result<Hash, VctrlError> {
        Hash::from_bytes(&[0_u8; HASH_LENGTH])
    }

    #[test]
    fn git_sort_tree_after_blob_with_same_prefix() -> Result<(), VctrlError> {
        let h = zero_hash()?;
        let blob_a = TreeEntry::new("a".into(), EntryKind::Blob, h)?;
        let tree_a = TreeEntry::new("a".into(), EntryKind::Tree, h)?;
        let blob_ab = TreeEntry::new("ab".into(), EntryKind::Blob, h)?;

        assert_eq!(compare_tree_entries(&blob_a, &tree_a), Ordering::Less);
        assert_eq!(compare_tree_entries(&tree_a, &blob_ab), Ordering::Less);

        Ok(())
    }

    #[test]
    fn tree_new_sorts_and_rejects_duplicates() -> Result<(), VctrlError> {
        let h = zero_hash()?;
        let e1 = TreeEntry::new("b".into(), EntryKind::Blob, h)?;
        let e2 = TreeEntry::new("a".into(), EntryKind::Blob, h)?;

        let tree = Tree::new(vec![e1, e2])?;
        assert_eq!(tree.entries().first().map(|e| e.name()), Some("a"));
        assert_eq!(tree.entries().get(1).map(|e| e.name()), Some("b"));

        let dup1 = TreeEntry::new("x".into(), EntryKind::Blob, h)?;
        let dup2 = TreeEntry::new("x".into(), EntryKind::Tree, h)?;
        assert!(Tree::new(vec![dup1, dup2]).is_err());

        Ok(())
    }
}
