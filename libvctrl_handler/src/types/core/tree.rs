//! Tree object representation.

use super::hash::Hash;
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

impl Tree {
    /// Creates a new tree from a list of entries.
    ///
    /// The entries must be sorted by name and contain no duplicates.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidName`] if entries are not sorted or contain duplicates.
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

    /// Returns the entries of the tree.
    #[must_use]
    pub fn entries(&self) -> &[TreeEntry] {
        &self.entries
    }
}
