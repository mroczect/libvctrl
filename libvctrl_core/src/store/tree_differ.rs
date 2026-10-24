use alloc::string::String;
use alloc::vec::Vec;
use std::path::Path;

use libvctrl_handler::{
    DiffEntry, DiffFileType, DiffStatus, EntryKind, Hash, Tree, TreeDiffer, VctrlError,
};

#[derive(Debug, Default)]
pub struct MemoryTreeDiffer {
    max_depth: usize,

    include_unmodified: bool,
}

impl MemoryTreeDiffer {
    pub fn new() -> Self {
        Self {
            max_depth: 100,
            include_unmodified: false,
        }
    }

    pub fn with_max_depth(depth: usize) -> Self {
        Self {
            max_depth: depth,
            include_unmodified: false,
        }
    }

    pub fn include_unmodified(mut self, include: bool) -> Self {
        self.include_unmodified = include;
        self
    }

    fn diff_entries(
        &self,
        old_entries: &[(String, EntryKind, Hash)],
        new_entries: &[(String, EntryKind, Hash)],
        prefix: &str,
        depth: usize,
        results: &mut Vec<DiffEntry>,
    ) -> Result<(), VctrlError> {
        if depth > self.max_depth {
            return Ok(());
        }

        let old_map: std::collections::BTreeMap<&str, &(String, EntryKind, Hash)> =
            old_entries.iter().map(|e| (e.0.as_str(), e)).collect();
        let new_map: std::collections::BTreeMap<&str, &(String, EntryKind, Hash)> =
            new_entries.iter().map(|e| (e.0.as_str(), e)).collect();

        let all_paths: std::collections::BTreeSet<&str> =
            old_map.keys().chain(new_map.keys()).copied().collect();

        for path in all_paths {
            let full_path = if prefix.is_empty() {
                path.to_string()
            } else {
                format!("{prefix}/{path}")
            };

            match (old_map.get(path), new_map.get(path)) {
                (Some(old), Some(new)) => {
                    if old.1 != new.1 {
                        results.push(DiffEntry::new(
                            full_path.clone(),
                            DiffStatus::TypeChange,
                            DiffFileType::from_entry_kind(old.1),
                            DiffFileType::from_entry_kind(new.1),
                            Some(*old.2),
                            Some(*new.2),
                        )?);
                    } else if old.2 != new.2 {
                        results.push(DiffEntry::new(
                            full_path.clone(),
                            DiffStatus::Modified,
                            DiffFileType::from_entry_kind(old.1),
                            DiffFileType::from_entry_kind(new.1),
                            Some(*old.2),
                            Some(*new.2),
                        )?);
                    } else if self.include_unmodified {
                        results.push(DiffEntry::new(
                            full_path.clone(),
                            DiffStatus::Unmodified,
                            DiffFileType::from_entry_kind(old.1),
                            DiffFileType::from_entry_kind(new.1),
                            Some(*old.2),
                            Some(*new.2),
                        )?);
                    }

                    if old.1 == EntryKind::Tree && new.1 == EntryKind::Tree {}
                }
                (Some(old), None) => {
                    results.push(DiffEntry::new(
                        full_path.clone(),
                        DiffStatus::Deleted,
                        DiffFileType::from_entry_kind(old.1),
                        DiffFileType::Absent,
                        Some(*old.2),
                        None,
                    )?);
                }
                (None, Some(new)) => {
                    results.push(DiffEntry::new(
                        full_path.clone(),
                        DiffStatus::Added,
                        DiffFileType::Absent,
                        DiffFileType::from_entry_kind(new.1),
                        None,
                        Some(*new.2),
                    )?);
                }
                (None, None) => unreachable!(),
            }
        }

        Ok(())
    }
}

impl TreeDiffer for MemoryTreeDiffer {
    fn diff_trees(&self, old_tree: &Tree, new_tree: &Tree) -> Result<Vec<DiffEntry>, VctrlError> {
        let old_entries: Vec<_> = old_tree
            .entries()
            .iter()
            .map(|e| (e.name().to_string(), e.kind(), *e.hash()))
            .collect();
        let new_entries: Vec<_> = new_tree
            .entries()
            .iter()
            .map(|e| (e.name().to_string(), e.kind(), *e.hash()))
            .collect();

        let mut results = Vec::new();
        self.diff_entries(&old_entries, &new_entries, "", 0, &mut results)?;
        Ok(results)
    }

    fn diff_tree_to_empty(&self, tree: &Tree) -> Result<Vec<DiffEntry>, VctrlError> {
        let empty = Tree::new(Vec::new())?;
        self.diff_trees(tree, &empty)
    }

    fn diff_empty_to_tree(&self, tree: &Tree) -> Result<Vec<DiffEntry>, VctrlError> {
        let empty = Tree::new(Vec::new())?;
        self.diff_trees(&empty, tree)
    }

    fn set_max_depth(&mut self, depth: usize) {
        self.max_depth = depth;
    }

    fn set_include_unmodified(&mut self, include: bool) {
        self.include_unmodified = include;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libvctrl_handler::TreeEntry;

    fn hash_byte(byte: u8) -> Hash {
        Hash::from_bytes(&[byte; 64]).unwrap()
    }

    fn make_tree(entries: Vec<(&str, EntryKind, u8)>) -> Tree {
        let entries: Vec<TreeEntry> = entries
            .into_iter()
            .map(|(name, kind, byte)| {
                TreeEntry::new(name.to_string(), kind, hash_byte(byte)).unwrap()
            })
            .collect();
        Tree::new(entries).unwrap()
    }

    #[test]
    fn diff_added_files() {
        let old = make_tree(vec![]);
        let new = make_tree(vec![("a.txt", EntryKind::Blob, 0x01)]);

        let differ = MemoryTreeDiffer::new();
        let diffs = differ.diff_empty_to_tree(&new).unwrap();

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path(), "a.txt");
        assert_eq!(diffs[0].status(), DiffStatus::Added);
    }

    #[test]
    fn diff_deleted_files() {
        let old = make_tree(vec![("a.txt", EntryKind::Blob, 0x01)]);
        let new = make_tree(vec![]);

        let differ = MemoryTreeDiffer::new();
        let diffs = differ.diff_tree_to_empty(&old).unwrap();

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].status(), DiffStatus::Deleted);
    }

    #[test]
    fn diff_modified_files() {
        let old = make_tree(vec![("a.txt", EntryKind::Blob, 0x01)]);
        let new = make_tree(vec![("a.txt", EntryKind::Blob, 0x02)]);

        let differ = MemoryTreeDiffer::new();
        let diffs = differ.diff_trees(&old, &new).unwrap();

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].status(), DiffStatus::Modified);
    }

    #[test]
    fn diff_type_change() {
        let old = make_tree(vec![("link", EntryKind::Symlink, 0x01)]);
        let new = make_tree(vec![("link", EntryKind::Blob, 0x01)]);

        let differ = MemoryTreeDiffer::new();
        let diffs = differ.diff_trees(&old, &new).unwrap();

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].status(), DiffStatus::TypeChange);
    }

    #[test]
    fn diff_unmodified_when_enabled() {
        let old = make_tree(vec![("a.txt", EntryKind::Blob, 0x01)]);
        let new = make_tree(vec![("a.txt", EntryKind::Blob, 0x01)]);

        let differ = MemoryTreeDiffer::new().include_unmodified(true);
        let diffs = differ.diff_trees(&old, &new).unwrap();

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].status(), DiffStatus::Unmodified);
    }

    #[test]
    fn diff_unmodified_when_disabled() {
        let old = make_tree(vec![("a.txt", EntryKind::Blob, 0x01)]);
        let new = make_tree(vec![("a.txt", EntryKind::Blob, 0x01)]);

        let differ = MemoryTreeDiffer::new();
        let diffs = differ.diff_trees(&old, &new).unwrap();

        assert!(diffs.is_empty());
    }

    #[test]
    fn mixed_changes() {
        let old = make_tree(vec![
            ("keep.txt", EntryKind::Blob, 0x01),
            ("modify.txt", EntryKind::Blob, 0x02),
            ("delete.txt", EntryKind::Blob, 0x03),
        ]);
        let new = make_tree(vec![
            ("keep.txt", EntryKind::Blob, 0x01),
            ("modify.txt", EntryKind::Blob, 0x99),
            ("add.txt", EntryKind::Blob, 0x04),
        ]);

        let differ = MemoryTreeDiffer::new();
        let mut diffs = differ.diff_trees(&old, &new).unwrap();
        diffs.sort_by(|a, b| a.path().cmp(b.path()));

        assert_eq!(diffs.len(), 3);
        assert_eq!(diffs[0].path(), "add.txt");
        assert_eq!(diffs[0].status(), DiffStatus::Added);
        assert_eq!(diffs[1].path(), "delete.txt");
        assert_eq!(diffs[1].status(), DiffStatus::Deleted);
        assert_eq!(diffs[2].path(), "modify.txt");
        assert_eq!(diffs[2].status(), DiffStatus::Modified);
    }
}
