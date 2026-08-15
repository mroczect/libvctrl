use super::hash::Hash;
use crate::constants::MAX_TREE_ENTRIES;
use crate::enums::EntryKind;
use crate::errors::VctrlError;
use crate::types::validate_tree_entry_name;
use std::cmp::Ordering;

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

fn entry_cmp(a: &TreeEntry, b: &TreeEntry) -> Ordering {
    let a_name = a.name().as_bytes();
    let b_name = b.name().as_bytes();
    let a_is_tree = a.kind() == EntryKind::Tree;
    let b_is_tree = b.kind() == EntryKind::Tree;

    let len = a_name.len().min(b_name.len());
    for i in 0..len {
        if a_name[i] != b_name[i] {
            return a_name[i].cmp(&b_name[i]);
        }
    }

    match a_name.len().cmp(&b_name.len()) {
        Ordering::Equal => Ordering::Equal,
        Ordering::Less => {
            if a_is_tree {
                b_name[len].cmp(&b'/')
            } else {
                Ordering::Less
            }
        }
        Ordering::Greater => {
            if b_is_tree {
                a_name[len].cmp(&b'/')
            } else {
                Ordering::Greater
            }
        }
    }
}

impl Tree {
    pub fn new(entries: Vec<TreeEntry>) -> Result<Self, VctrlError> {
        let max_entries = usize::try_from(MAX_TREE_ENTRIES).unwrap_or(usize::MAX);
        if entries.len() > max_entries {
            return Err(VctrlError::ExceededMaxSize(format!(
                "tree entries count {} exceeds maximum allowed count {MAX_TREE_ENTRIES}",
                entries.len()
            )));
        }

        for i in 1..entries.len() {
            let prev = &entries[i - 1];
            let curr = &entries[i];
            if entry_cmp(prev, curr) != Ordering::Less {
                return Err(VctrlError::InvalidTreeStructure(format!(
                    "Tree entries are not sorted or contain duplicates: '{}' vs '{}'",
                    prev.name(),
                    curr.name()
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
