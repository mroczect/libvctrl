mod common;
use common::blob_hash;

use libvctrl::{DiffKind, EntryKind, Tree, TreeDiff, TreeDiffer, TreeEntry};

#[test]
fn diff_added_removed_modified() {
    let h1 = blob_hash(b"one");
    let h2 = blob_hash(b"two");
    let h3 = blob_hash(b"three");

    let old = Tree::new(vec![
        TreeEntry::new("a".into(), EntryKind::Blob, h1),
        TreeEntry::new("b".into(), EntryKind::Blob, h2),
    ])
    .unwrap();

    let new = Tree::new(vec![
        TreeEntry::new("a".into(), EntryKind::Blob, h3),
        TreeEntry::new("c".into(), EntryKind::Blob, h1),
    ])
    .unwrap();

    let differ = TreeDiffer;
    let diffs = differ.diff(&old, &new).unwrap();
    assert_eq!(diffs.len(), 3);

    let added = diffs
        .iter()
        .find(|d| matches!(d.kind, DiffKind::Added))
        .unwrap();
    assert_eq!(added.name, "c");

    let removed = diffs
        .iter()
        .find(|d| matches!(d.kind, DiffKind::Removed))
        .unwrap();
    assert_eq!(removed.name, "b");

    let modified = diffs
        .iter()
        .find(|d| matches!(d.kind, DiffKind::Modified { .. }))
        .unwrap();
    assert_eq!(modified.name, "a");
    if let DiffKind::Modified { old_hash, new_hash } = modified.kind {
        assert_eq!(old_hash, h1);
        assert_eq!(new_hash, h3);
    } else {
        panic!("expected Modified");
    }
}

#[test]
fn diff_no_changes() {
    let h = blob_hash(b"same");
    let tree = Tree::new(vec![TreeEntry::new("f".into(), EntryKind::Blob, h)]).unwrap();
    let differ = TreeDiffer;
    let diffs = differ.diff(&tree, &tree).unwrap();
    assert!(diffs.is_empty());
}
