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

#[cfg(test)]
mod tests {
    use super::*;

    fn hash_byte(byte: u8) -> Result<Hash, VctrlError> {
        Hash::from_bytes(&[byte; 64])
    }

    #[test]
    fn tree_entry_builder_valid() -> Result<(), VctrlError> {
        let hash = hash_byte(0x11)?;
        let entry = TreeEntryBuilder::new("file.txt".to_string(), EntryKind::Blob, hash).build()?;
        assert_eq!(entry.name(), "file.txt");
        assert_eq!(entry.kind(), EntryKind::Blob);
        assert_eq!(*entry.hash(), hash);
        Ok(())
    }

    #[test]
    fn tree_entry_builder_empty_name_errors() -> Result<(), VctrlError> {
        let hash = hash_byte(0x11)?;
        let result = TreeEntryBuilder::new(String::new(), EntryKind::Blob, hash).build();
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn tree_builder_add_entry_and_build() -> Result<(), VctrlError> {
        let hash = hash_byte(0x22)?;
        let tree = TreeBuilder::new()
            .add_entry("a".to_string(), EntryKind::Blob, hash)?
            .build()?;

        let entries = tree.entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries
                .first()
                .ok_or_else(|| VctrlError::Other("expected entry".into()))?
                .name(),
            "a"
        );
        Ok(())
    }

    #[test]
    fn tree_builder_build_empty_tree() -> Result<(), VctrlError> {
        let tree = TreeBuilder::new().build()?;
        assert!(tree.entries().is_empty());
        Ok(())
    }
}
