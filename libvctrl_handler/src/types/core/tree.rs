use super::hash::Hash;
use crate::constants::MAX_TREE_ENTRIES;
use crate::enums::EntryKind;
use crate::errors::VctrlError;
use crate::validation::validate_tree_entry_name;
use std::cmp::Ordering;

/// A single entry in a Git tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeEntry {
    name: String,
    kind: EntryKind,
    hash: Hash,
}

impl TreeEntry {
    /// Creates a new tree entry.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidName`] if the entry name is invalid.
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
    /// # Errors
    ///
    /// Returns [`VctrlError::ExceededMaxSize`] if the entry count exceeds `MAX_TREE_ENTRIES`.
    /// Returns [`VctrlError::InvalidTreeStructure`] if duplicate names are found.
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
            if window[0].name == window[1].name {
                return Err(VctrlError::InvalidTreeStructure(format!(
                    "duplicate entry name: '{}'",
                    window[0].name
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
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&TreeEntry> {
        self.entries.iter().find(|e| e.name == name)
    }
}

/// Compares two tree entries using Git ordering rules.
///
/// Tree entries (directories) are compared as if their name has a
/// trailing `/` appended. All other kinds use their name as-is.
#[inline]
fn compare_tree_entries(a: &TreeEntry, b: &TreeEntry) -> Ordering {
    let a_bytes = a.name.as_bytes();
    let b_bytes = b.name.as_bytes();
    let a_is_tree = a.kind == EntryKind::Tree;
    let b_is_tree = b.kind == EntryKind::Tree;

    // Effective length: tree entries get an extra byte for the trailing '/'
    let a_len = a_bytes.len() + usize::from(a_is_tree);
    let b_len = b_bytes.len() + usize::from(b_is_tree);
    let min_len = a_len.min(b_len);

    for i in 0..min_len {
        let a_byte = if i < a_bytes.len() { a_bytes[i] } else { b'/' };
        let b_byte = if i < b_bytes.len() { b_bytes[i] } else { b'/' };
        match a_byte.cmp(&b_byte) {
            Ordering::Equal => {}
            ord => return ord,
        }
    }

    a_len.cmp(&b_len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::HASH_LENGTH;

    fn zero_hash() -> Hash {
        Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap()
    }

    #[test]
    fn git_sort_tree_after_blob_with_same_prefix() {
        let h = zero_hash();
        let blob_a = TreeEntry::new("a".into(), EntryKind::Blob, h).unwrap();
        let tree_a = TreeEntry::new("a".into(), EntryKind::Tree, h).unwrap();
        let blob_ab = TreeEntry::new("ab".into(), EntryKind::Blob, h).unwrap();

        // Git order: a (blob) < a/ (tree) < ab (blob)
        assert_eq!(compare_tree_entries(&blob_a, &tree_a), Ordering::Less);
        assert_eq!(compare_tree_entries(&tree_a, &blob_ab), Ordering::Less);
    }

    #[test]
    fn tree_new_sorts_and_rejects_duplicates() {
        let h = zero_hash();
        let e1 = TreeEntry::new("b".into(), EntryKind::Blob, h).unwrap();
        let e2 = TreeEntry::new("a".into(), EntryKind::Blob, h).unwrap();

        let tree = Tree::new(vec![e1, e2]).unwrap();
        assert_eq!(tree.entries()[0].name(), "a");
        assert_eq!(tree.entries()[1].name(), "b");

        let dup1 = TreeEntry::new("x".into(), EntryKind::Blob, h).unwrap();
        let dup2 = TreeEntry::new("x".into(), EntryKind::Tree, h).unwrap();
        assert!(Tree::new(vec![dup1, dup2]).is_err());
    }
}
