mod common;
use common::*;
use libvctrl::*;

#[test]
fn index_add_remove_to_tree() {
    let mut idx = Index::new();
    let h1 = blob_hash(b"a");
    let h2 = blob_hash(b"b");
    let e1 = TreeEntry::new("f1".into(), EntryKind::Blob, h1).unwrap();
    let e2 = TreeEntry::new("f2".into(), EntryKind::Blob, h2).unwrap();

    idx.add(e1.clone());
    idx.add(e2.clone());
    assert_eq!(idx.iter().count(), 2);

    let removed = idx.remove("f1");
    assert!(removed.is_some());
    let removed_entry = removed.unwrap();
    assert_eq!(removed_entry.name, "f1");
    assert_eq!(removed_entry.hash, h1);
    assert_eq!(idx.iter().count(), 1);

    let tree = idx.into_tree().unwrap();
    assert_eq!(tree.entries().len(), 1);
    assert_eq!(tree.entries()[0].name, "f2");
}
