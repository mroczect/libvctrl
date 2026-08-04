mod common;
use common::setup_store;

use libvctrl::{Blob, EntryKind, Object, ObjectStore, Tree, TreeEntry, VctrlError, merge_trees};

#[test]
fn merge_no_conflict_simple() {
    let mut store = setup_store();

    let a1 = store.put(&Object::Blob(Blob::new(b"1".to_vec()))).unwrap();
    let b2 = store.put(&Object::Blob(Blob::new(b"2".to_vec()))).unwrap();
    let base_tree = Tree::new(vec![
        TreeEntry::new("a".into(), EntryKind::Blob, a1),
        TreeEntry::new("b".into(), EntryKind::Blob, b2),
    ])
    .unwrap();
    let base_hash = store.put(&Object::Tree(base_tree)).unwrap();

    let b3 = store.put(&Object::Blob(Blob::new(b"3".to_vec()))).unwrap();
    let ours_tree = Tree::new(vec![
        TreeEntry::new("a".into(), EntryKind::Blob, a1),
        TreeEntry::new("b".into(), EntryKind::Blob, b3),
    ])
    .unwrap();
    let ours_hash = store.put(&Object::Tree(ours_tree)).unwrap();

    let c4 = store.put(&Object::Blob(Blob::new(b"4".to_vec()))).unwrap();
    let theirs_tree = Tree::new(vec![
        TreeEntry::new("a".into(), EntryKind::Blob, a1),
        TreeEntry::new("b".into(), EntryKind::Blob, b2),
        TreeEntry::new("c".into(), EntryKind::Blob, c4),
    ])
    .unwrap();
    let theirs_hash = store.put(&Object::Tree(theirs_tree)).unwrap();

    let resolver = |_: &[u8], _: &[u8], _: &[u8]| -> Option<Vec<u8>> { None };

    let merged_hash =
        merge_trees(&mut store, &base_hash, &ours_hash, &theirs_hash, &resolver).unwrap();
    let merged_tree = match store.get(&merged_hash).unwrap() {
        Some(Object::Tree(t)) => t,
        _ => panic!("expected tree"),
    };

    let mut entries = merged_tree.into_entries();
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].name, "a");
    assert_eq!(entries[0].hash, a1);
    assert_eq!(entries[1].name, "b");
    assert_eq!(entries[1].hash, b3);
    assert_eq!(entries[2].name, "c");
    assert_eq!(entries[2].hash, c4);
}

#[test]
fn merge_conflict_blob() {
    let mut store = setup_store();

    let base_data = store
        .put(&Object::Blob(Blob::new(b"base".to_vec())))
        .unwrap();
    let ours_data = store
        .put(&Object::Blob(Blob::new(b"ours".to_vec())))
        .unwrap();
    let theirs_data = store
        .put(&Object::Blob(Blob::new(b"theirs".to_vec())))
        .unwrap();

    let base_tree =
        Tree::new(vec![TreeEntry::new("f".into(), EntryKind::Blob, base_data)]).unwrap();
    let base_hash = store.put(&Object::Tree(base_tree)).unwrap();

    let ours_tree =
        Tree::new(vec![TreeEntry::new("f".into(), EntryKind::Blob, ours_data)]).unwrap();
    let ours_hash = store.put(&Object::Tree(ours_tree)).unwrap();

    let theirs_tree = Tree::new(vec![TreeEntry::new(
        "f".into(),
        EntryKind::Blob,
        theirs_data,
    )])
    .unwrap();
    let theirs_hash = store.put(&Object::Tree(theirs_tree)).unwrap();

    let resolver = |_: &[u8], _: &[u8], _: &[u8]| -> Option<Vec<u8>> { None };
    let err = merge_trees(&mut store, &base_hash, &ours_hash, &theirs_hash, &resolver).unwrap_err();
    match err {
        VctrlError::MergeConflict { entry, .. } => assert_eq!(entry, "f"),
        _ => panic!("expected MergeConflict"),
    }
}

#[test]
fn merge_resolved() {
    let mut store = setup_store();

    let base_data = store
        .put(&Object::Blob(Blob::new(b"base".to_vec())))
        .unwrap();
    let ours_data = store
        .put(&Object::Blob(Blob::new(b"ours".to_vec())))
        .unwrap();
    let theirs_data = store
        .put(&Object::Blob(Blob::new(b"theirs".to_vec())))
        .unwrap();

    let base_tree =
        Tree::new(vec![TreeEntry::new("f".into(), EntryKind::Blob, base_data)]).unwrap();
    let base_hash = store.put(&Object::Tree(base_tree)).unwrap();

    let ours_tree =
        Tree::new(vec![TreeEntry::new("f".into(), EntryKind::Blob, ours_data)]).unwrap();
    let ours_hash = store.put(&Object::Tree(ours_tree)).unwrap();

    let theirs_tree = Tree::new(vec![TreeEntry::new(
        "f".into(),
        EntryKind::Blob,
        theirs_data,
    )])
    .unwrap();
    let theirs_hash = store.put(&Object::Tree(theirs_tree)).unwrap();

    let resolver = |_: &[u8], _: &[u8], _: &[u8]| -> Option<Vec<u8>> { Some(b"resolved".to_vec()) };

    let merged_hash =
        merge_trees(&mut store, &base_hash, &ours_hash, &theirs_hash, &resolver).unwrap();
    let merged_tree = match store.get(&merged_hash).unwrap() {
        Some(Object::Tree(t)) => t,
        _ => panic!("expected tree"),
    };
    let entries = merged_tree.into_entries();
    assert_eq!(entries.len(), 1);
    let blob = match store.get(&entries[0].hash).unwrap() {
        Some(Object::Blob(b)) => b,
        _ => panic!("expected blob"),
    };
    assert_eq!(blob.into_bytes(), b"resolved");
}
