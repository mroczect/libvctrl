mod common;
use common::setup_store;

use libvctrl::{Blob, EntryKind, Object, ObjectStore, Tree, TreeEntry, VctrlError, checkout_tree};

#[test]
fn checkout_flat_tree() {
    let mut store = setup_store();
    let h1 = store
        .put(&Object::Blob(Blob::new(b"data1".to_vec())))
        .unwrap();
    let h2 = store
        .put(&Object::Blob(Blob::new(b"data2".to_vec())))
        .unwrap();

    let tree = Tree::new(vec![
        TreeEntry::new("a.txt".into(), EntryKind::Blob, h1),
        TreeEntry::new("b.txt".into(), EntryKind::Blob, h2),
    ])
    .unwrap();
    let tree_hash = store.put(&Object::Tree(tree)).unwrap();

    let files = checkout_tree(&store, &tree_hash).unwrap();
    assert_eq!(files.len(), 2);
    let mut paths: Vec<&str> = files.iter().map(|(path, _)| path.as_str()).collect();
    paths.sort();
    assert_eq!(paths, vec!["a.txt", "b.txt"]);
}

#[test]
fn checkout_recursive() {
    let mut store = setup_store();
    let h1 = store
        .put(&Object::Blob(Blob::new(b"inner".to_vec())))
        .unwrap();

    let sub_tree = Tree::new(vec![TreeEntry::new(
        "inner.txt".into(),
        EntryKind::Blob,
        h1,
    )])
    .unwrap();
    let sub_hash = store.put(&Object::Tree(sub_tree)).unwrap();

    let root_tree = Tree::new(vec![TreeEntry::new(
        "sub".into(),
        EntryKind::Tree,
        sub_hash,
    )])
    .unwrap();
    let root_hash = store.put(&Object::Tree(root_tree)).unwrap();

    let files = checkout_tree(&store, &root_hash).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].0, "sub/inner.txt");
    assert_eq!(files[0].1, b"inner");
}

#[test]
fn checkout_empty_tree() {
    let mut store = setup_store();
    let tree = Tree::new(vec![]).unwrap();
    let hash = store.put(&Object::Tree(tree)).unwrap();
    let files = checkout_tree(&store, &hash).unwrap();
    assert!(files.is_empty());
}

#[test]
fn checkout_nonexistent_tree_error() {
    let store = setup_store();
    let fake_hash = Blob::new(b"dummy".to_vec()).hash().unwrap();
    let err = checkout_tree(&store, &fake_hash).unwrap_err();
    match err {
        VctrlError::NotFound(_) => {}
        _ => panic!("expected NotFound"),
    }
}
