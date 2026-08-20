













use super::hash::Hash;
use crate::constants::MAX_TREE_ENTRIES;
use crate::enums::EntryKind;
use crate::errors::VctrlError;
use crate::validation::validate_tree_entry_name;
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

impl Tree {
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    pub fn new(entries: Vec<TreeEntry>) -> Result<Self, VctrlError> {
        let max_entries = usize::try_from(MAX_TREE_ENTRIES).unwrap_or(usize::MAX);
        if entries.len() > max_entries {
            return Err(VctrlError::ExceededMaxSize(format!(
                "tree has {} entries, exceeding maximum of {MAX_TREE_ENTRIES}",
                entries.len()
            )));
        }

        let mut sorted = entries;
        sorted.sort_by(compare_tree_entries);

        for window in sorted.windows(2) {
            if let (Some(first), Some(second)) = (window.first(), window.get(1))
                && first.name == second.name
            {
                return Err(VctrlError::InvalidTreeStructure(format!(
                    "duplicate entry name: '{}'",
                    first.name
                )));
            }
        }

        Ok(Self { entries: sorted })
    }

    
    #[must_use]
    pub fn entries(&self) -> &[TreeEntry] {
        &self.entries
    }

    
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    
    
    
    
    
    
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&TreeEntry> {
        self.entries.iter().find(|e| e.name == name)
    }
}










#[inline]
fn compare_tree_entries(a: &TreeEntry, b: &TreeEntry) -> Ordering {
    let a_bytes = a.name.as_bytes();
    let b_bytes = b.name.as_bytes();
    let a_is_tree = a.kind == EntryKind::Tree;
    let b_is_tree = b.kind == EntryKind::Tree;

    let a_len = a_bytes.len() + usize::from(a_is_tree);
    let b_len = b_bytes.len() + usize::from(b_is_tree);
    let min_len = a_len.min(b_len);

    for i in 0..min_len {
        let a_byte = a_bytes.get(i).copied().unwrap_or(b'/');
        let b_byte = b_bytes.get(i).copied().unwrap_or(b'/');
        match a_byte.cmp(&b_byte) {
            Ordering::Equal => {}
            Ordering::Less => return Ordering::Less,
            Ordering::Greater => return Ordering::Greater,
        }
    }

    a_len.cmp(&b_len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::HASH_LENGTH;

    fn zero_hash() -> Result<Hash, VctrlError> {
        Hash::from_bytes(&[0_u8; HASH_LENGTH])
    }

    #[test]
    fn git_sort_tree_after_blob_with_same_prefix() -> Result<(), VctrlError> {
        let h = zero_hash()?;
        let blob_a = TreeEntry::new("a".into(), EntryKind::Blob, h)?;
        let tree_a = TreeEntry::new("a".into(), EntryKind::Tree, h)?;
        let blob_ab = TreeEntry::new("ab".into(), EntryKind::Blob, h)?;

        assert_eq!(compare_tree_entries(&blob_a, &tree_a), Ordering::Less);
        assert_eq!(compare_tree_entries(&tree_a, &blob_ab), Ordering::Less);

        Ok(())
    }

    #[test]
    fn tree_new_sorts_and_rejects_duplicates() -> Result<(), VctrlError> {
        let h = zero_hash()?;
        let e1 = TreeEntry::new("b".into(), EntryKind::Blob, h)?;
        let e2 = TreeEntry::new("a".into(), EntryKind::Blob, h)?;

        let tree = Tree::new(vec![e1, e2])?;
        assert_eq!(tree.entries().first().map(|e| e.name()), Some("a"));
        assert_eq!(tree.entries().get(1).map(|e| e.name()), Some("b"));

        let dup1 = TreeEntry::new("x".into(), EntryKind::Blob, h)?;
        let dup2 = TreeEntry::new("x".into(), EntryKind::Tree, h)?;
        assert!(Tree::new(vec![dup1, dup2]).is_err());

        Ok(())
    }
}
