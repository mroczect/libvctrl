mod common;
use common::*;
use libvctrl::*;

#[test]
fn patch_roundtrip_blob_only() {
    let mut store = MemoryStore::new();
    let h1 = put_blob(&mut store, b"old");
    let h2 = put_blob(&mut store, b"new");

    let old_tree = Tree::new(vec![
        TreeEntry::new("a".into(), EntryKind::Blob, h1).unwrap(),
    ])
    .unwrap();
    let new_tree = Tree::new(vec![
        TreeEntry::new("a".into(), EntryKind::Blob, h2).unwrap(),
    ])
    .unwrap();

    let patch_data = generate_patch(&old_tree, &new_tree).unwrap();
    let applied = apply_patch(&old_tree, &patch_data, &mut store, &hasher()).unwrap();
    let entries = applied.entries();
    assert_eq!(entries[0].hash, h2);
}

#[test]
fn patch_rejects_tree_entries() {
    let tree_hash1 = blob_hash(b"t1");
    let tree_hash2 = blob_hash(b"t2");

    let old = Tree::new(vec![
        TreeEntry::new("dir".into(), EntryKind::Tree, tree_hash1).unwrap(),
    ])
    .unwrap();
    let new = Tree::new(vec![
        TreeEntry::new("dir".into(), EntryKind::Tree, tree_hash2).unwrap(),
    ])
    .unwrap();

    assert!(generate_patch(&old, &new).is_err());
}

#[test]
fn apply_patch_conflict_detection() {
    let mut store = MemoryStore::new();
    let h1 = put_blob(&mut store, b"v1");
    let h2 = put_blob(&mut store, b"v2");
    let h3 = put_blob(&mut store, b"v3");

    let base = Tree::new(vec![
        TreeEntry::new("f".into(), EntryKind::Blob, h1).unwrap(),
    ])
    .unwrap();
    let modified_base = Tree::new(vec![
        TreeEntry::new("f".into(), EntryKind::Blob, h3).unwrap(),
    ])
    .unwrap();

    let new = Tree::new(vec![
        TreeEntry::new("f".into(), EntryKind::Blob, h2).unwrap(),
    ])
    .unwrap();
    let patch = generate_patch(&base, &new).unwrap();
    let err = apply_patch(&modified_base, &patch, &mut store, &hasher()).unwrap_err();
    assert!(err.to_string().contains("has been modified"));
}
