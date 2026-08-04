mod common;
use common::{blob_hash, put_blob, put_tree, setup_refs, setup_store};

use libvctrl::{Checkout, Command, EntryKind, Tree, TreeEntry, VctrlError};

#[test]
fn checkout_flat_tree() {
    let mut store = setup_store();
    let mut refs = setup_refs();
    let h1 = put_blob(&mut store, b"data1");
    let h2 = put_blob(&mut store, b"data2");

    let tree = Tree::new(vec![
        TreeEntry::new("a.txt".into(), EntryKind::Blob, h1).unwrap(),
        TreeEntry::new("b.txt".into(), EntryKind::Blob, h2).unwrap(),
    ])
    .unwrap();
    let tree_hash = put_tree(&mut store, &tree);

    let cmd = Checkout { tree_hash };
    let files = cmd.execute(&mut store, &mut refs).unwrap();
    assert_eq!(files.len(), 2);
    let mut paths: Vec<&str> = files.iter().map(|(p, _)| p.as_str()).collect();
    paths.sort();
    assert_eq!(paths, vec!["a.txt", "b.txt"]);
}

#[test]
fn checkout_recursive() {
    let mut store = setup_store();
    let mut refs = setup_refs();
    let inner_hash = put_blob(&mut store, b"inner");

    let sub_tree = Tree::new(vec![
        TreeEntry::new("inner.txt".into(), EntryKind::Blob, inner_hash).unwrap(),
    ])
    .unwrap();
    let sub_hash = put_tree(&mut store, &sub_tree);

    let root_tree = Tree::new(vec![
        TreeEntry::new("sub".into(), EntryKind::Tree, sub_hash).unwrap(),
    ])
    .unwrap();
    let root_hash = put_tree(&mut store, &root_tree);

    let cmd = Checkout {
        tree_hash: root_hash,
    };
    let files = cmd.execute(&mut store, &mut refs).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].0, "sub/inner.txt");
    assert_eq!(files[0].1, b"inner");
}

#[test]
fn checkout_empty_tree() {
    let mut store = setup_store();
    let mut refs = setup_refs();
    let tree = Tree::new(vec![]).unwrap();
    let hash = put_tree(&mut store, &tree);

    let cmd = Checkout { tree_hash: hash };
    let files = cmd.execute(&mut store, &mut refs).unwrap();
    assert!(files.is_empty());
}

#[test]
fn checkout_nonexistent_tree_error() {
    let mut store = setup_store();
    let mut refs = setup_refs();
    let fake_hash = blob_hash(b"dummy");

    let cmd = Checkout {
        tree_hash: fake_hash,
    };
    let err = cmd.execute(&mut store, &mut refs).unwrap_err();
    match err {
        VctrlError::NotFound(_) => {}
        _ => panic!("expected NotFound"),
    }
}
