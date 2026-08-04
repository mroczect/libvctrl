use libvctrl::*;
use std::fs;

#[test]
fn test_file_store_basic() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.vctl");
    if path.exists() {
        fs::remove_file(&path).unwrap();
    }

    let mut store = FileStore::open(&path).unwrap();

    let blob = Blob::new(b"hello".to_vec());
    let hash = Sha512Hasher.hash_blob(b"hello");
    store.put(&hash, &Object::Blob(blob)).unwrap();

    let obj = store.get(&hash).unwrap().unwrap();
    assert!(matches!(obj, Object::Blob(_)));
    if let Object::Blob(b) = obj {
        assert_eq!(b.as_bytes(), b"hello");
    }

    store.set_ref("refs/heads/main", &hash).unwrap();
    assert_eq!(store.get_ref("refs/heads/main").unwrap(), Some(hash));

    store.set_head("refs/heads/main").unwrap();
    assert_eq!(store.head().unwrap(), Some(hash));
}
