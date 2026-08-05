use crate::domain::tree::{Tree, TreeEntry, TreeError};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct Index {
    entries: BTreeMap<String, TreeEntry>,
}

impl Index {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, entry: TreeEntry) {
        self.entries.insert(entry.name.clone(), entry);
    }

    pub fn remove(&mut self, name: &str) -> Option<TreeEntry> {
        self.entries.remove(name)
    }

    pub fn get(&self, name: &str) -> Option<&TreeEntry> {
        self.entries.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = &TreeEntry> {
        self.entries.values()
    }

    pub fn to_tree(&self) -> Result<Tree, TreeError> {
        let entries: Vec<_> = self.entries.values().cloned().collect();
        Tree::new(entries)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
