mod common;
use common::*;
use libvctrl::*;

#[test]
fn gc_memory_store() {
    let mut store = MemoryStore::new();
    let mut refs = MemoryRefStore::new();

    let blob_hash = put_blob(&mut store, b"data");
    let tree = Tree::new(vec![
        TreeEntry::new("file.txt".into(), EntryKind::Blob, blob_hash).unwrap(),
    ])
    .unwrap();
    let tree_hash = put_tree(&mut store, &tree);
    let commit = Commit::new(tree_hash, vec![], alice(), alice(), "msg".into(), None);
    let c_hash = commit_hash(&commit);
    store
        .put(&c_hash, &Object::Commit(Box::new(commit)))
        .unwrap();

    refs.set_ref("refs/heads/main", &c_hash).unwrap();
    refs.set_head("refs/heads/main").unwrap();

    let unreachable = put_blob(&mut store, b"unreachable");

    let removed = gc::gc(&mut store, &refs).unwrap();
    assert_eq!(removed, 1);
    assert!(!store.exists(&unreachable).unwrap());
    assert!(store.exists(&blob_hash).unwrap());
}

#[test]
fn gc_file_store() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("gc.vctl");
    let _ = std::fs::remove_file(&path);

    let mut store = FileStore::open(&path).unwrap();
    let mut refs = MemoryRefStore::new();

    let h1 = Sha512Hasher.hash_blob(b"reachable");
    let h2 = Sha512Hasher.hash_blob(b"garbage");
    store
        .put(&h1, &Object::Blob(Blob::new(b"r".to_vec())))
        .unwrap();
    store
        .put(&h2, &Object::Blob(Blob::new(b"g".to_vec())))
        .unwrap();

    refs.set_ref("refs/heads/main", &h1).unwrap();
    refs.set_head("refs/heads/main").unwrap();

    let removed = gc::gc(&mut store, &refs).unwrap();
    assert_eq!(removed, 1);
    assert!(!store.exists(&h2).unwrap());
}
