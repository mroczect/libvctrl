mod common;
use common::{encoder, hasher, put_blob, put_tree, setup_refs, setup_store};

use libvctrl::{
    Command, ConflictResolver, EntryKind, MergeCommand, Object, ObjectStore, ThreeWayMerger, Tree,
    TreeEntry, VctrlError,
};

struct SimpleResolver;
impl ConflictResolver for SimpleResolver {
    fn resolve(&self, _base: &[u8], _ours: &[u8], _theirs: &[u8]) -> Option<Vec<u8>> {
        None
    }
}

struct KeepOursResolver;
impl ConflictResolver for KeepOursResolver {
    fn resolve(&self, _base: &[u8], ours: &[u8], _theirs: &[u8]) -> Option<Vec<u8>> {
        Some(ours.to_vec())
    }
}

#[test]
fn merge_no_conflict() {
    let mut store = setup_store();
    let mut refs = setup_refs();
    let a1 = put_blob(&mut store, b"1");
    let b2 = put_blob(&mut store, b"2");

    let base_tree = Tree::new(vec![
        TreeEntry::new("a".into(), EntryKind::Blob, a1),
        TreeEntry::new("b".into(), EntryKind::Blob, b2),
    ])
    .unwrap();
    let base_hash = put_tree(&mut store, &base_tree);

    let b3 = put_blob(&mut store, b"3");
    let ours_tree = Tree::new(vec![
        TreeEntry::new("a".into(), EntryKind::Blob, a1),
        TreeEntry::new("b".into(), EntryKind::Blob, b3),
    ])
    .unwrap();
    let ours_hash = put_tree(&mut store, &ours_tree);

    let c4 = put_blob(&mut store, b"4");
    let theirs_tree = Tree::new(vec![
        TreeEntry::new("a".into(), EntryKind::Blob, a1),
        TreeEntry::new("b".into(), EntryKind::Blob, b2),
        TreeEntry::new("c".into(), EntryKind::Blob, c4),
    ])
    .unwrap();
    let theirs_hash = put_tree(&mut store, &theirs_tree);

    let cmd = MergeCommand {
        base: base_hash,
        ours: ours_hash,
        theirs: theirs_hash,
        merger: Box::new(ThreeWayMerger),
        resolver: Box::new(SimpleResolver),
        encoder: Box::new(encoder()),
        hasher: Box::new(hasher()),
    };
    let merged_hash = cmd.execute(&mut store, &mut refs).unwrap();
    let obj = store.get(&merged_hash).unwrap().unwrap();
    let tree = match obj {
        Object::Tree(t) => t,
        _ => panic!("expected tree"),
    };

    let mut entries = tree.into_entries();
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].hash, a1);
    assert_eq!(entries[1].hash, b3);
    assert_eq!(entries[2].hash, c4);
}

#[test]
fn merge_conflict_blob() {
    let mut store = setup_store();
    let mut refs = setup_refs();
    let base_blob = put_blob(&mut store, b"base");
    let ours_blob = put_blob(&mut store, b"ours");
    let theirs_blob = put_blob(&mut store, b"theirs");

    let base_tree =
        Tree::new(vec![TreeEntry::new("f".into(), EntryKind::Blob, base_blob)]).unwrap();
    let base_hash = put_tree(&mut store, &base_tree);
    let ours_tree =
        Tree::new(vec![TreeEntry::new("f".into(), EntryKind::Blob, ours_blob)]).unwrap();
    let ours_hash = put_tree(&mut store, &ours_tree);
    let theirs_tree = Tree::new(vec![TreeEntry::new(
        "f".into(),
        EntryKind::Blob,
        theirs_blob,
    )])
    .unwrap();
    let theirs_hash = put_tree(&mut store, &theirs_tree);

    let cmd = MergeCommand {
        base: base_hash,
        ours: ours_hash,
        theirs: theirs_hash,
        merger: Box::new(ThreeWayMerger),
        resolver: Box::new(SimpleResolver),
        encoder: Box::new(encoder()),
        hasher: Box::new(hasher()),
    };
    let err = cmd.execute(&mut store, &mut refs).unwrap_err();
    match err {
        VctrlError::MergeConflict { entry, .. } => assert_eq!(entry, "f"),
        _ => panic!("expected MergeConflict"),
    }
}

#[test]
fn merge_resolved() {
    let mut store = setup_store();
    let mut refs = setup_refs();
    let base_blob = put_blob(&mut store, b"base");
    let ours_blob = put_blob(&mut store, b"ours");
    let theirs_blob = put_blob(&mut store, b"theirs");

    let base_tree =
        Tree::new(vec![TreeEntry::new("f".into(), EntryKind::Blob, base_blob)]).unwrap();
    let base_hash = put_tree(&mut store, &base_tree);
    let ours_tree =
        Tree::new(vec![TreeEntry::new("f".into(), EntryKind::Blob, ours_blob)]).unwrap();
    let ours_hash = put_tree(&mut store, &ours_tree);
    let theirs_tree = Tree::new(vec![TreeEntry::new(
        "f".into(),
        EntryKind::Blob,
        theirs_blob,
    )])
    .unwrap();
    let theirs_hash = put_tree(&mut store, &theirs_tree);

    let cmd = MergeCommand {
        base: base_hash,
        ours: ours_hash,
        theirs: theirs_hash,
        merger: Box::new(ThreeWayMerger),
        resolver: Box::new(KeepOursResolver),
        encoder: Box::new(encoder()),
        hasher: Box::new(hasher()),
    };
    let merged_hash = cmd.execute(&mut store, &mut refs).unwrap();
    let obj = store.get(&merged_hash).unwrap().unwrap();
    let tree = match obj {
        Object::Tree(t) => t,
        _ => panic!("expected tree"),
    };
    let entries = tree.into_entries();
    assert_eq!(entries.len(), 1);
    let blob = match store.get(&entries[0].hash).unwrap().unwrap() {
        Object::Blob(b) => b,
        _ => panic!("expected blob"),
    };
    assert_eq!(blob.into_bytes(), b"ours");
}
