//! Tree object representation.

use super::hash::Hash;
use crate::constants::MAX_TREE_ENTRIES;
use crate::enums::EntryKind;
use crate::errors::VctrlError;
use crate::types::validate_tree_entry_name;

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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tree {
    entries: Vec<TreeEntry>,
}

fn entry_sort_key(name: &str, kind: EntryKind) -> Vec<u8> {
    let mut key = name.as_bytes().to_vec();
    if kind == EntryKind::Tree {
        key.push(b'/');
    }
    key
}

impl Tree {
    /// Creates a new tree from a list of entries.
    ///
    /// The entries must be sorted according to Git's rules and contain no duplicates.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidTreeStructure`] if entries are not sorted or contain duplicates.
    /// Returns [`VctrlError::ExceededMaxSize`] if the number of entries exceeds `MAX_TREE_ENTRIES`.
    pub fn new(entries: Vec<TreeEntry>) -> Result<Self, VctrlError> {
        if entries.len() > MAX_TREE_ENTRIES as usize {
            return Err(VctrlError::ExceededMaxSize(format!(
                "tree entries count {} exceeds maximum allowed count {MAX_TREE_ENTRIES}",
                entries.len()
            )));
        }

        for i in 1..entries.len() {
            let prev_key = entry_sort_key(entries[i - 1].name(), entries[i - 1].kind());
            let curr_key = entry_sort_key(entries[i].name(), entries[i].kind());
            if prev_key >= curr_key {
                return Err(VctrlError::InvalidTreeStructure(format!(
                    "Tree entries are not sorted or contain duplicates: '{}' vs '{}'",
                    entries[i - 1].name(),
                    entries[i].name()
                )));
            }
        }
        Ok(Self { entries })
    }

    /// Returns the entries of the tree.
    #[must_use]
    pub fn entries(&self) -> &[TreeEntry] {
        &self.entries
    }
}
