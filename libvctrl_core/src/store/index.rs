use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use libvctrl_handler::{
    Decoder, Encoder, EntryKind, Hash, Hasher, Index, ObjectStore, Tree, TreeEntry, VctrlError,
};

use crate::codec::BinaryEncoder;
use crate::hash::Sha512Hasher;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexEntry {
    path: String,
    kind: EntryKind,
    hash: Hash,
}

impl IndexEntry {
    pub fn new(path: impl Into<String>, kind: EntryKind, hash: Hash) -> Result<Self, VctrlError> {
        let path = path.into();
        if path.is_empty() {
            return Err(VctrlError::InvalidName("path cannot be empty".into()));
        }
        if path.contains('\0') {
            return Err(VctrlError::InvalidName("path contains null byte".into()));
        }
        if path.starts_with('/') {
            return Err(VctrlError::InvalidName("path must be relative".into()));
        }
        Ok(Self { path, kind, hash })
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
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

#[derive(Debug, Clone, Default)]
pub struct MemoryIndex {
    entries: BTreeMap<String, IndexEntry>,
    dirty: bool,
}

impl MemoryIndex {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            dirty: false,
        }
    }

    pub fn add_entry(
        &mut self,
        path: impl AsRef<str>,
        kind: EntryKind,
        hash: Hash,
    ) -> Result<(), VctrlError> {
        let path = Self::normalize_path(path.as_ref());
        let entry = IndexEntry::new(path, kind, hash)?;
        Index::add(self, entry)
    }

    pub fn remove_entry(&mut self, path: impl AsRef<str>) -> Result<(), VctrlError> {
        let path = Self::normalize_path(path.as_ref());
        Index::remove(self, &path)
    }

    #[must_use]
    pub fn get_entry(&self, path: impl AsRef<str>) -> Option<&IndexEntry> {
        let path = Self::normalize_path(path.as_ref());
        self.entries.get(&path)
    }

    #[must_use]
    pub fn entries_sorted(&self) -> Vec<&IndexEntry> {
        let mut entries: Vec<_> = self.entries.values().collect();
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        entries
    }

    #[must_use]
    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub const fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    fn normalize_path(path: &str) -> String {
        path.replace('\\', "/").trim_end_matches('/').to_string()
    }
}

impl Index for MemoryIndex {
    type Entry = IndexEntry;
    type Path = String;
    type TreeId = Hash;

    fn add(&mut self, entry: Self::Entry) -> Result<(), VctrlError> {
        let _ = self.entries.insert(entry.path.clone(), entry);
        self.dirty = true;
        Ok(())
    }

    fn remove(&mut self, path: &Self::Path) -> Result<(), VctrlError> {
        let _ = self.entries.remove(path);
        self.dirty = true;
        Ok(())
    }

    fn clear(&mut self) -> Result<(), VctrlError> {
        self.entries.clear();
        self.dirty = true;
        Ok(())
    }

    fn get(&self, path: &Self::Path) -> Result<Option<Self::Entry>, VctrlError> {
        Ok(self.entries.get(path).cloned())
    }

    fn contains(&self, path: &Self::Path) -> Result<bool, VctrlError> {
        Ok(self.entries.contains_key(path))
    }

    fn len(&self) -> Result<usize, VctrlError> {
        Ok(self.entries.len())
    }

    fn entries(&self) -> Result<Vec<Self::Entry>, VctrlError> {
        Ok(self.entries.values().cloned().collect())
    }

    fn write_tree(&self) -> Result<Self::TreeId, VctrlError> {
        let mut tree_entries: Vec<TreeEntry> = self
            .entries
            .values()
            .map(|e| TreeEntry::new(e.path.clone(), e.kind, e.hash))
            .collect::<Result<_, _>>()?;

        tree_entries.sort_by(|a, b| a.name().cmp(b.name()));
        let tree = Tree::new(tree_entries)?;

        let encoder = BinaryEncoder;
        let mut buf = Vec::new();
        encoder.encode_tree(&tree, &mut buf)?;

        let hasher = Sha512Hasher;
        hasher.hash(buf.as_slice())
    }

    fn read_tree<D: Decoder>(
        &mut self,
        store: &dyn ObjectStore,
        decoder: &D,
        tree: &Hash,
    ) -> Result<(), VctrlError> {
        use std::io::Read;

        let mut reader = store.get(tree)?;
        let mut encoded = Vec::new();
        let _ = reader.read_to_end(&mut encoded)?;

        let tree = decoder.decode_tree(encoded.as_slice())?;

        self.entries.clear();
        for entry in tree.entries() {
            let idx = IndexEntry::new(entry.name().to_string(), entry.kind(), *entry.hash())?;
            let _ = self.entries.insert(entry.name().to_string(), idx);
        }
        self.dirty = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash_byte(byte: u8) -> Result<Hash, VctrlError> {
        Hash::from_bytes(&[byte; 64])
    }

    #[test]
    fn add_and_get_entry() -> Result<(), VctrlError> {
        let mut index = MemoryIndex::new();
        let hash = hash_byte(0x01)?;

        index.add_entry("src/main.rs", EntryKind::Blob, hash)?;

        let entry = index
            .get_entry("src/main.rs")
            .ok_or_else(|| VctrlError::Other("missing entry".into()))?;
        assert_eq!(entry.path(), "src/main.rs");
        assert_eq!(entry.kind(), EntryKind::Blob);
        assert_eq!(*entry.hash(), hash);
        Ok(())
    }

    #[test]
    fn path_normalization() -> Result<(), VctrlError> {
        let mut index = MemoryIndex::new();
        index.add_entry("foo/bar/", EntryKind::Blob, hash_byte(0x01)?)?;
        assert!(index.get_entry("foo/bar").is_some());
        assert!(index.get_entry("foo/bar/").is_some());
        Ok(())
    }

    #[test]
    fn remove_entry() -> Result<(), VctrlError> {
        let mut index = MemoryIndex::new();
        index.add_entry("file.txt", EntryKind::Blob, hash_byte(0x01)?)?;

        index.remove_entry("file.txt")?;
        assert!(index.get_entry("file.txt").is_none());
        Ok(())
    }

    #[test]
    fn reject_empty_path() -> Result<(), VctrlError> {
        let mut index = MemoryIndex::new();
        assert!(
            index
                .add_entry("", EntryKind::Blob, hash_byte(0x01)?)
                .is_err()
        );
        Ok(())
    }

    #[test]
    fn reject_absolute_path() -> Result<(), VctrlError> {
        let mut index = MemoryIndex::new();
        assert!(
            index
                .add_entry("/etc/passwd", EntryKind::Blob, hash_byte(0x01)?)
                .is_err()
        );
        Ok(())
    }

    #[test]
    fn entries_sorted() -> Result<(), VctrlError> {
        let mut index = MemoryIndex::new();
        index.add_entry("z.txt", EntryKind::Blob, hash_byte(0x01)?)?;
        index.add_entry("a.txt", EntryKind::Blob, hash_byte(0x02)?)?;
        index.add_entry("m.txt", EntryKind::Blob, hash_byte(0x03)?)?;

        let sorted = index.entries_sorted();
        assert_eq!(sorted.first().map(|e| e.path()), Some("a.txt"));
        assert_eq!(sorted.get(1).map(|e| e.path()), Some("m.txt"));
        assert_eq!(sorted.get(2).map(|e| e.path()), Some("z.txt"));
        Ok(())
    }

    #[test]
    fn len_and_empty() -> Result<(), VctrlError> {
        let mut index = MemoryIndex::new();
        assert!(index.is_empty()?);
        assert_eq!(index.len()?, 0);

        index.add_entry("a", EntryKind::Blob, hash_byte(0x01)?)?;
        assert!(!index.is_empty()?);
        assert_eq!(index.len()?, 1);
        Ok(())
    }

    #[test]
    fn dirty_tracking() -> Result<(), VctrlError> {
        let mut index = MemoryIndex::new();
        assert!(!index.is_dirty());

        index.add_entry("file.txt", EntryKind::Blob, hash_byte(0x01)?)?;
        assert!(index.is_dirty());

        index.clear_dirty();
        assert!(!index.is_dirty());
        Ok(())
    }

    #[test]
    fn clear_index() -> Result<(), VctrlError> {
        let mut index = MemoryIndex::new();
        index.add_entry("a", EntryKind::Blob, hash_byte(0x01)?)?;
        index.add_entry("b", EntryKind::Blob, hash_byte(0x02)?)?;

        index.clear()?;
        assert!(index.is_empty()?);
        Ok(())
    }

    #[test]
    fn trait_methods() -> Result<(), VctrlError> {
        let mut index = MemoryIndex::new();
        let hash = hash_byte(0x01)?;
        let entry = IndexEntry::new("file.txt", EntryKind::Blob, hash)?;
        index.add(entry.clone())?;

        assert!(index.contains(&"file.txt".to_string())?);
        let got = index
            .get(&"file.txt".to_string())?
            .ok_or_else(|| VctrlError::Other("entry missing".into()))?;
        assert_eq!(got, entry);

        let entries = index.entries()?;
        assert_eq!(entries.len(), 1);

        let tree_hash = index.write_tree()?;
        assert_eq!(tree_hash.as_bytes().len(), 64);

        Ok(())
    }
}
