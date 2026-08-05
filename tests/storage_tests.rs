mod common;
use common::*;
use libvctrl::*;
use std::fs;

#[test]
fn memory_store_all_hashes_and_remove() {
    let mut store = MemoryStore::new();
    let h1 = blob_hash(b"a");
    let h2 = blob_hash(b"b");
    store
        .put(&h1, &Object::Blob(Blob::new(b"a".to_vec())))
        .unwrap();
    store
        .put(&h2, &Object::Blob(Blob::new(b"b".to_vec())))
        .unwrap();

    let mut hashes = store.all_hashes().unwrap();
    hashes.sort_by_key(|h| h.to_hex());
    assert_eq!(hashes, vec![h1, h2]);

    store.remove(&h1).unwrap();
    let hashes = store.all_hashes().unwrap();
    assert_eq!(hashes, vec![h2]);
    assert!(store.get(&h1).unwrap().is_none());
}

#[test]
fn file_store_remove_and_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.vctl");
    let _ = fs::remove_file(&path);

    let mut store = FileStore::open(&path).unwrap();
    let h1 = Sha512Hasher.hash_blob(b"keep");
    let h2 = Sha512Hasher.hash_blob(b"delete");

    store
        .put(&h1, &Object::Blob(Blob::new(b"keep".to_vec())))
        .unwrap();
    store
        .put(&h2, &Object::Blob(Blob::new(b"delete".to_vec())))
        .unwrap();

    assert!(store.exists(&h2).unwrap());
    store.remove(&h2).unwrap();
    assert!(!store.exists(&h2).unwrap());
    assert!(store.get(&h2).unwrap().is_none());

    drop(store);
    let store2 = FileStore::open(&path).unwrap();
    assert!(store2.exists(&h1).unwrap());
    assert!(!store2.exists(&h2).unwrap());
    let hashes = store2.all_hashes().unwrap();
    assert_eq!(hashes, vec![h1]);
}
