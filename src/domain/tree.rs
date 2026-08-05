use super::hash::Hash;
use serde::{Deserialize, Serialize};

pub const MAX_TREE_DEPTH: usize = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryKind {
    Blob,
    Tree,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeEntry {
    pub name: String,
    pub kind: EntryKind,
    pub hash: Hash,
}

pub const MAX_ENTRY_NAME_LEN: usize = 255;

impl TreeEntry {
    pub fn new(name: String, kind: EntryKind, hash: Hash) -> Result<Self, TreeError> {
        if name.len() > MAX_ENTRY_NAME_LEN {
            return Err(TreeError::InvalidEntryName(
                "name exceeds maximum length (255)".into(),
            ));
        }
        if name.is_empty()
            || name.contains('/')
            || name.contains('\\')
            || name == ".."
            || name == "."
        {
            return Err(TreeError::InvalidEntryName(name));
        }
        Ok(Self { name, kind, hash })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tree {
    entries: Vec<TreeEntry>,
}

impl Tree {
    pub fn new(mut entries: Vec<TreeEntry>) -> Result<Self, TreeError> {
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        for pair in entries.windows(2) {
            if pair[0].name == pair[1].name {
                return Err(TreeError::DuplicateEntry(pair[0].name.clone()));
            }
        }
        Ok(Self { entries })
    }
    pub fn entries(&self) -> &[TreeEntry] {
        &self.entries
    }
    pub fn into_entries(self) -> Vec<TreeEntry> {
        self.entries
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum TreeError {
    #[error("duplicate entry name: {0}")]
    DuplicateEntry(String),
    #[error("invalid entry name: {0}")]
    InvalidEntryName(String),
}
