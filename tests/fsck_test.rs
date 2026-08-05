mod common;
use common::*;
use libvctrl::*;

#[test]
fn fsck_no_errors() {
    let mut store = MemoryStore::new();
    let mut refs = MemoryRefStore::new();

    let blob = put_blob(&mut store, b"ok");
    let tree = Tree::new(vec![
        TreeEntry::new("f".into(), EntryKind::Blob, blob).unwrap(),
    ])
    .unwrap();
    let tree_h = put_tree(&mut store, &tree);

    let commit = Commit::new(tree_h, vec![], alice(), alice(), "ok".into(), None);
    let c_hash = commit_hash(&commit);
    store
        .put(&c_hash, &Object::Commit(Box::new(commit)))
        .unwrap();

    let cmd = Fsck {
        encoder: Box::new(encoder()),
        hasher: Box::new(hasher()),
    };
    let errors = cmd.execute(&mut store, &mut refs).unwrap();
    assert!(errors.is_empty());
}

#[test]
fn fsck_detects_corruption() {
    let mut store = MemoryStore::new();
    let mut refs = MemoryRefStore::new();

    let original_data = b"original";
    let hash = hasher().hash_blob(original_data);

    let corrupted_blob = Blob::new(b"corrupted".to_vec());
    store.put(&hash, &Object::Blob(corrupted_blob)).unwrap();

    let cmd = Fsck {
        encoder: Box::new(encoder()),
        hasher: Box::new(hasher()),
    };
    let errors = cmd.execute(&mut store, &mut refs).unwrap();
    assert_eq!(errors.len(), 1);
    assert!(matches!(errors[0], VctrlError::Corrupted(_)));
}
