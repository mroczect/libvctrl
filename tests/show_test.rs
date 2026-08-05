mod common;
use common::*;
use libvctrl::*;

#[test]
fn show_commit_with_diff() {
    let mut store = MemoryStore::new();
    let mut refs = MemoryRefStore::new();

    let base_blob = put_blob(&mut store, b"old");
    let new_blob = put_blob(&mut store, b"new");

    let parent_tree = Tree::new(vec![
        TreeEntry::new("f".into(), EntryKind::Blob, base_blob).unwrap(),
    ])
    .unwrap();
    let parent_hash = put_tree(&mut store, &parent_tree);
    let parent_commit = Commit::new(parent_hash, vec![], alice(), alice(), "parent".into(), None);
    let parent_h = commit_hash(&parent_commit);
    store
        .put(&parent_h, &Object::Commit(Box::new(parent_commit)))
        .unwrap();

    let child_tree = Tree::new(vec![
        TreeEntry::new("f".into(), EntryKind::Blob, new_blob).unwrap(),
    ])
    .unwrap();
    let child_hash = put_tree(&mut store, &child_tree);
    let child_commit = Commit::new(
        child_hash,
        vec![parent_h],
        alice(),
        alice(),
        "child".into(),
        None,
    );
    let child_h = commit_hash(&child_commit);
    store
        .put(&child_h, &Object::Commit(Box::new(child_commit)))
        .unwrap();

    let show = Show {
        commit_hash: child_h,
    };
    let output = show.execute(&mut store, &mut refs).unwrap();
    assert_eq!(output.commit.message, "child");
    let diff = output.diff.unwrap();
    assert_eq!(diff.len(), 1);
    assert!(matches!(diff[0].kind, DiffKind::Modified { .. }));
}
