use libvctrl_handler::{EntryKind, Hash, Tree, TreeEntry, VctrlError};

#[derive(Debug, Default)]
pub struct TreeBuilder {
    entries: Vec<TreeEntry>,
}

impl TreeBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    #[must_use]
    pub fn entry(mut self, entry: TreeEntry) -> Self {
        self.entries.push(entry);
        self
    }

    pub fn add_entry(
        mut self,
        name: String,
        kind: EntryKind,
        hash: Hash,
    ) -> Result<Self, VctrlError> {
        let entry = TreeEntry::new(name, kind, hash)?;
        self.entries.push(entry);
        Ok(self)
    }

    pub fn build(self) -> Result<Tree, VctrlError> {
        Tree::new(self.entries)
    }
}

#[derive(Debug)]
pub struct TreeEntryBuilder {
    name: String,
    kind: EntryKind,
    hash: Hash,
}

impl TreeEntryBuilder {
    #[must_use]
    pub const fn new(name: String, kind: EntryKind, hash: Hash) -> Self {
        Self { name, kind, hash }
    }

    pub fn build(self) -> Result<TreeEntry, VctrlError> {
        TreeEntry::new(self.name, self.kind, self.hash)
    }
}
