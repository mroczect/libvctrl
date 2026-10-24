use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use std::io;

use libvctrl_handler::{Hash, ReflogEntry, ReflogStore, VctrlError};

#[derive(Debug, Clone)]
pub struct MemoryReflogStore {
    logs: BTreeMap<String, Vec<ReflogEntry>>,

    max_entries: usize,
}

impl Default for MemoryReflogStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryReflogStore {
    pub fn new() -> Self {
        Self {
            logs: BTreeMap::new(),
            max_entries: 1000,
        }
    }

    pub fn with_max_entries(max: usize) -> Self {
        Self {
            logs: BTreeMap::new(),
            max_entries: max,
        }
    }

    fn trim_entries(&mut self, ref_name: &str) {
        if let Some(entries) = self.logs.get_mut(ref_name) {
            while entries.len() > self.max_entries {
                entries.pop();
            }
        }
    }
}

impl ReflogStore for MemoryReflogStore {
    fn append(
        &mut self,
        ref_name: impl Into<String>,
        old_hash: Option<Hash>,
        new_hash: Hash,
        committer_name: impl Into<String>,
        committer_email: impl Into<String>,
        message: impl Into<String>,
    ) -> Result<(), VctrlError> {
        let ref_name = ref_name.into();
        let entry = ReflogEntry::new(
            old_hash,
            new_hash,
            committer_name.into(),
            committer_email.into(),
            message.into(),
        )?;

        self.logs
            .entry(ref_name.clone())
            .or_default()
            .insert(0, entry);

        self.trim_entries(&ref_name);
        Ok(())
    }

    fn get(&self, ref_name: &str) -> Result<Vec<&ReflogEntry>, VctrlError> {
        Ok(self
            .logs
            .get(ref_name)
            .map(|v| v.iter().collect())
            .unwrap_or_default())
    }

    fn get_entry(&self, ref_name: &str, index: usize) -> Result<Option<&ReflogEntry>, VctrlError> {
        Ok(self
            .logs
            .get(ref_name)
            .and_then(|entries| entries.get(index)))
    }

    fn drop_entry(&mut self, ref_name: &str, index: usize) -> Result<bool, VctrlError> {
        if let Some(entries) = self.logs.get_mut(ref_name) {
            if index < entries.len() {
                entries.remove(index);
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn rewrite(
        &mut self,
        ref_name: &str,
        old_hash: &Hash,
        new_hash: Hash,
    ) -> Result<usize, VctrlError> {
        let entries = self.logs.get_mut(ref_name);
        let entries = match entries {
            Some(e) => e,
            None => return Ok(0),
        };

        let mut count = 0;
        for entry in entries.iter_mut() {
            if entry.new_hash() == old_hash {
                entry.replace_new_hash(new_hash);
                count += 1;
            }
            if entry.old_hash() == Some(old_hash) {
                entry.replace_old_hash(new_hash);
            }
        }
        Ok(count)
    }

    fn delete(&mut self, ref_name: &str) -> Result<bool, VctrlError> {
        Ok(self.logs.remove(ref_name).is_some())
    }

    fn has_reflog(&self, ref_name: &str) -> bool {
        self.logs.contains_key(ref_name)
    }

    fn ref_names(&self) -> Vec<&str> {
        self.logs.keys().map(|s| s.as_str()).collect()
    }

    fn len(&self, ref_name: &str) -> usize {
        self.logs.get(ref_name).map(|v| v.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash_byte(byte: u8) -> Hash {
        Hash::from_bytes(&[byte; 64]).unwrap()
    }

    #[test]
    fn append_and_get() {
        let mut store = MemoryReflogStore::new();

        store
            .append(
                "HEAD",
                None,
                hash_byte(0x01),
                "Alice",
                "alice@example.com",
                "commit: initial",
            )
            .unwrap();

        let entries = store.get("HEAD").unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].old_hash().is_none());
        assert_eq!(*entries[0].new_hash(), hash_byte(0x01));
    }

    #[test]
    fn multiple_entries_ordered_newest_first() {
        let mut store = MemoryReflogStore::new();

        store
            .append(
                "HEAD",
                Some(hash_byte(0x01)),
                hash_byte(0x02),
                "A",
                "a@b",
                "msg1",
            )
            .unwrap();
        store
            .append(
                "HEAD",
                Some(hash_byte(0x02)),
                hash_byte(0x03),
                "A",
                "a@b",
                "msg2",
            )
            .unwrap();

        let entries = store.get("HEAD").unwrap();
        assert_eq!(*entries[0].new_hash(), hash_byte(0x03));
        assert_eq!(*entries[1].new_hash(), hash_byte(0x02));
    }

    #[test]
    fn get_entry_by_index() {
        let mut store = MemoryReflogStore::new();
        store
            .append("main", None, hash_byte(0x01), "A", "a@b", "first")
            .unwrap();
        store
            .append(
                "main",
                Some(hash_byte(0x01)),
                hash_byte(0x02),
                "A",
                "a@b",
                "second",
            )
            .unwrap();

        let entry = store.get_entry("main", 1).unwrap().unwrap();
        assert_eq!(entry.message(), "first");
    }

    #[test]
    fn drop_entry() {
        let mut store = MemoryReflogStore::new();
        store
            .append("HEAD", None, hash_byte(0x01), "A", "a@b", "msg")
            .unwrap();

        assert!(store.drop_entry("HEAD", 0).unwrap());
        assert!(!store.drop_entry("HEAD", 0).unwrap());
        assert!(store.get("HEAD").unwrap().is_empty());
    }

    #[test]
    fn rewrite_entries() {
        let mut store = MemoryReflogStore::new();
        store
            .append("HEAD", None, hash_byte(0x01), "A", "a@b", "msg")
            .unwrap();
        store
            .append(
                "HEAD",
                Some(hash_byte(0x01)),
                hash_byte(0x02),
                "A",
                "a@b",
                "msg",
            )
            .unwrap();

        let count = store
            .rewrite("HEAD", &hash_byte(0x01), hash_byte(0x99))
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn delete_reflog() {
        let mut store = MemoryReflogStore::new();
        store
            .append("HEAD", None, hash_byte(0x01), "A", "a@b", "msg")
            .unwrap();

        assert!(store.delete("HEAD").unwrap());
        assert!(!store.delete("HEAD").unwrap());
        assert!(!store.has_reflog("HEAD"));
    }

    #[test]
    fn max_entries_trimming() {
        let mut store = MemoryReflogStore::with_max_entries(2);

        store
            .append("HEAD", None, hash_byte(0x01), "A", "a@b", "1")
            .unwrap();
        store
            .append(
                "HEAD",
                Some(hash_byte(0x01)),
                hash_byte(0x02),
                "A",
                "a@b",
                "2",
            )
            .unwrap();
        store
            .append(
                "HEAD",
                Some(hash_byte(0x02)),
                hash_byte(0x03),
                "A",
                "a@b",
                "3",
            )
            .unwrap();

        assert_eq!(store.len("HEAD"), 2);
        let entries = store.get("HEAD").unwrap();
        assert_eq!(*entries[0].new_hash(), hash_byte(0x03));
        assert_eq!(*entries[1].new_hash(), hash_byte(0x02));
    }

    #[test]
    fn ref_names_listing() {
        let mut store = MemoryReflogStore::new();
        store
            .append("HEAD", None, hash_byte(0x01), "A", "a@b", "")
            .unwrap();
        store
            .append("main", None, hash_byte(0x02), "A", "a@b", "")
            .unwrap();
        store
            .append("feature", None, hash_byte(0x03), "A", "a@b", "")
            .unwrap();

        let mut names: Vec<&str> = store.ref_names();
        names.sort();
        assert_eq!(names, vec!["HEAD", "feature", "main"]);
    }
}
