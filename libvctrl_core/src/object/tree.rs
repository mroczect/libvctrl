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
    use libvctrl_handler::HASH_LENGTH;

    fn make_hash(fill: u8) -> Hash {
        Hash::from_bytes(&vec![fill; HASH_LENGTH]).unwrap()
    }

    #[test]
    fn test_tree_builder_build_empty() {
        let result = TreeBuilder::new().build();
        assert!(result.is_ok(), "empty tree should be valid");
        let tree = result.unwrap();
        assert_eq!(tree.entries().len(), 0);
    }

    #[test]
    fn test_tree_builder_build_with_entries_via_entry_method() {
        let entry = TreeEntry::new("README.md".into(), EntryKind::Blob, make_hash(0x01)).unwrap();
        let result = TreeBuilder::new().entry(entry).build();
        assert!(result.is_ok());
        let tree = result.unwrap();
        assert_eq!(tree.entries().len(), 1);
        assert_eq!(tree.entries()[0].name(), "README.md");
    }

    #[test]
    fn test_tree_builder_build_with_multiple_entries() {
        let e1 = TreeEntry::new("src".into(), EntryKind::Tree, make_hash(0x01)).unwrap();
        let e2 = TreeEntry::new("Cargo.toml".into(), EntryKind::Blob, make_hash(0x02)).unwrap();
        let result = TreeBuilder::new().entry(e1).entry(e2).build();
        assert!(result.is_ok());
        let tree = result.unwrap();
        assert_eq!(tree.entries().len(), 2);
    }

    #[test]
    fn test_tree_builder_add_entry_success() {
        let result = TreeBuilder::new()
            .add_entry("main.rs".into(), EntryKind::Blob, make_hash(0x03))
            .and_then(|b| b.build());
        assert!(result.is_ok());
        let tree = result.unwrap();
        assert_eq!(tree.entries()[0].name(), "main.rs");
        assert_eq!(tree.entries()[0].kind(), EntryKind::Blob);
    }

    #[test]
    fn test_tree_builder_add_entry_chaining() {
        let result = TreeBuilder::new()
            .add_entry("a.txt".into(), EntryKind::Blob, make_hash(0x10))
            .and_then(|b| b.add_entry("b.txt".into(), EntryKind::Blob, make_hash(0x20)))
            .and_then(|b| b.build());
        assert!(result.is_ok());
        let tree = result.unwrap();
        assert_eq!(tree.entries().len(), 2);
    }

    #[test]
    fn test_tree_entry_builder_build_success() {
        let result =
            TreeEntryBuilder::new("lib.rs".into(), EntryKind::Blob, make_hash(0x42)).build();
        assert!(result.is_ok());
        let entry = result.unwrap();
        assert_eq!(entry.name(), "lib.rs");
        assert_eq!(entry.kind(), EntryKind::Blob);
    }

    #[test]
    fn test_tree_entry_builder_all_kinds() {
        for kind in [
            EntryKind::Blob,
            EntryKind::Executable,
            EntryKind::Symlink,
            EntryKind::Tree,
            EntryKind::Submodule,
        ] {
            let result =
                TreeEntryBuilder::new(format!("item_{kind:?}"), kind, make_hash(0xFF)).build();
            assert!(result.is_ok(), "should succeed for kind {kind:?}");
            assert_eq!(result.unwrap().kind(), kind);
        }
    }
}
