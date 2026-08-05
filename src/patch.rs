use crate::diff::{DiffKind, TreeDiff, TreeDiffer};
use crate::domain::hash::Hash;
use crate::domain::tree::{EntryKind, Tree, TreeEntry};
use crate::error::VctrlError;
use crate::hashing::Hasher;
use crate::storage::traits::ObjectStore;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::collections::BTreeMap;
use std::io::Cursor;
use std::io::Read;

pub fn generate_patch(old_tree: &Tree, new_tree: &Tree) -> Result<Vec<u8>, VctrlError> {
    let differ = TreeDiffer;
    let diffs = differ.diff(old_tree, new_tree)?;

    let mut buf = Vec::new();
    buf.write_u32::<BigEndian>(diffs.len() as u32).unwrap();

    for d in &diffs {
        match &d.kind {
            DiffKind::Added { new_hash } => {
                buf.write_u8(0u8).unwrap();
                write_diff_entry_name(&d.name, &mut buf);
                buf.extend_from_slice(new_hash.as_bytes());
            }
            DiffKind::Removed => {
                buf.write_u8(1u8).unwrap();
                write_diff_entry_name(&d.name, &mut buf);
            }
            DiffKind::Modified { old_hash, new_hash } => {
                buf.write_u8(2u8).unwrap();
                write_diff_entry_name(&d.name, &mut buf);
                buf.extend_from_slice(old_hash.as_bytes());
                buf.extend_from_slice(new_hash.as_bytes());
            }
        }
    }
    Ok(buf)
}

fn write_diff_entry_name(name: &str, buf: &mut Vec<u8>) {
    let bytes = name.as_bytes();
    buf.write_u16::<BigEndian>(bytes.len() as u16).unwrap();
    buf.extend_from_slice(bytes);
}

pub fn apply_patch(
    base_tree: &Tree,
    patch_data: &[u8],
    _store: &mut dyn ObjectStore,
    _hasher: &dyn Hasher,
) -> Result<Tree, VctrlError> {
    let mut cursor = Cursor::new(patch_data);
    let count = cursor
        .read_u32::<BigEndian>()
        .map_err(|e| VctrlError::Other(e.to_string()))?;
    let mut entries: BTreeMap<String, TreeEntry> = base_tree
        .entries()
        .iter()
        .map(|e| (e.name.clone(), e.clone()))
        .collect();

    for _ in 0..count {
        let kind = cursor
            .read_u8()
            .map_err(|e| VctrlError::Other(e.to_string()))?;
        let name_len = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| VctrlError::Other(e.to_string()))?;
        let mut name_bytes = vec![0u8; name_len as usize];
        cursor
            .read_exact(&mut name_bytes)
            .map_err(|e| VctrlError::Other(e.to_string()))?;
        let name = String::from_utf8(name_bytes).map_err(|e| VctrlError::Other(e.to_string()))?;

        match kind {
            0 => {
                let mut hash_bytes = [0u8; 64];
                cursor
                    .read_exact(&mut hash_bytes)
                    .map_err(|e| VctrlError::Other(e.to_string()))?;
                let hash = Hash::from_bytes(hash_bytes);
                let entry = TreeEntry::new(name.clone(), EntryKind::Blob, hash)
                    .map_err(VctrlError::Tree)?;
                entries.insert(name, entry);
            }
            1 => {
                entries.remove(&name);
            }
            2 => {
                let mut old_hash_bytes = [0u8; 64];
                cursor
                    .read_exact(&mut old_hash_bytes)
                    .map_err(|e| VctrlError::Other(e.to_string()))?;
                let mut new_hash_bytes = [0u8; 64];
                cursor
                    .read_exact(&mut new_hash_bytes)
                    .map_err(|e| VctrlError::Other(e.to_string()))?;
                let new_hash = Hash::from_bytes(new_hash_bytes);
                let old_hash = Hash::from_bytes(old_hash_bytes);
                if let Some(existing) = entries.get(&name)
                    && existing.hash != old_hash
                {
                    return Err(VctrlError::Other(format!(
                        "patch conflict: '{}' has been modified",
                        name
                    )));
                }
                let entry = TreeEntry::new(name.clone(), EntryKind::Blob, new_hash)
                    .map_err(VctrlError::Tree)?;
                entries.insert(name, entry);
            }
            _ => return Err(VctrlError::Other("invalid patch kind".into())),
        }
    }
    let entries: Vec<_> = entries.into_values().collect();
    Tree::new(entries).map_err(VctrlError::Tree)
}
