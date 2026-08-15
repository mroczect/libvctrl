use super::hash::Hash;
use crate::enums::EntryKind;
use crate::errors::VctrlError;
use crate::types::validate_tree_entry_name;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeEntry {
    name: String,
    kind: EntryKind,
    hash: Hash,
}

impl TreeEntry {
    pub fn new(name: String, kind: EntryKind, hash: Hash) -> Result<Self, VctrlError> {
        validate_tree_entry_name(&name)?;
        Ok(Self { name, kind, hash })
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn kind(&self) -> EntryKind {
        self.kind
    }

    #[must_use]
    pub const fn hash(&self) -> &Hash {
        &self.hash
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tree {
    entries: Vec<TreeEntry>,
}

impl Tree {
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

    #[must_use]
    pub fn entries(&self) -> &[TreeEntry] {
        &self.entries
    }
}
