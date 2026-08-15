use libvctrl_core::hash::Sha512Hasher;
use libvctrl_core::store::{MemoryRefStore, MemoryStore};
use libvctrl_handler::{Hash, Hasher, MAX_NAME_LENGTH, ObjectStore, RefStore};
use std::io::Read;

fn dummy_hash_from_data(data: &[u8]) -> Hash {
    let hasher = Sha512Hasher;
    hasher.hash(data).unwrap()
}

#[test]
fn test_memory_store_crud_and_streaming() {
    let mut store = MemoryStore::new();
    let data = b"hello world";
    let hash = dummy_hash_from_data(data);

    // Put
    store.put(&hash, data).unwrap();

    // Exists
    assert!(store.exists(&hash).unwrap());
    assert!(!store.exists(&dummy_hash_from_data(b"other")).unwrap());

    // Get and verify (zero-clone streaming)
    {
        let mut reader = store.get(&hash).unwrap();
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).unwrap();
        assert_eq!(buf, data);
    } // reader is dropped here, releasing the immutable borrow

    // Delete
    store.delete(&hash).unwrap();
    assert!(!store.exists(&hash).unwrap());

    // Delete non-existent
    assert!(store.delete(&hash).is_ok());

    // Get non-existent
    assert!(store.get(&hash).is_err());
}

#[test]
fn test_memory_store_large_object_streaming() {
    let mut store = MemoryStore::new();
    // 10 MB object to test zero-copy cursor limits
    let data = vec![0x42u8; 10 * 1024 * 1024];
    let hash = dummy_hash_from_data(&data);

    store.put(&hash, &data).unwrap();

    let mut reader = store.get(&hash).unwrap();
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).unwrap();

    assert_eq!(buf.len(), data.len());
    assert_eq!(buf, data);
}

#[test]
fn test_memory_ref_store_crud_and_sorting() {
    let mut store = MemoryRefStore::new();
    let hash1 = dummy_hash_from_data(b"1");
    let hash2 = dummy_hash_from_data(b"2");

    // Set refs
    store.set_ref("refs/heads/main", &hash1).unwrap();
    store.set_ref("refs/heads/feature", &hash2).unwrap();

    // Get
    assert_eq!(store.get_ref("refs/heads/main").unwrap(), hash1);

    // List (should be sorted)
    let refs: Vec<String> = store
        .list_refs()
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(refs, vec!["refs/heads/feature", "refs/heads/main"]);

    // Delete
    store.delete_ref("refs/heads/main").unwrap();
    assert!(store.get_ref("refs/heads/main").is_err());

    let refs: Vec<String> = store
        .list_refs()
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(refs, vec!["refs/heads/feature"]);
}

#[test]
fn test_memory_ref_store_strict_validation() {
    let mut store = MemoryRefStore::new();
    let hash = dummy_hash_from_data(b"1");

    // Empty name
    assert!(store.set_ref("", &hash).is_err());

    // Too long name
    let long_name = "a".repeat(usize::try_from(MAX_NAME_LENGTH).unwrap() + 1);
    assert!(store.set_ref(&long_name, &hash).is_err());

    // Path traversal attempts (Security)
    assert!(store.set_ref("../config", &hash).is_err());
    assert!(store.set_ref("..\\config", &hash).is_err());
    assert!(store.set_ref("refs/heads/..", &hash).is_err());

    // Git illegal characters
    assert!(store.set_ref("refs/heads/foo~bar", &hash).is_err());
    assert!(store.set_ref("refs/heads/foo^bar", &hash).is_err());
    assert!(store.set_ref("refs/heads/foo:bar", &hash).is_err());
    assert!(store.set_ref("refs/heads/foo bar", &hash).is_err()); // Space
    assert!(store.set_ref("refs/heads/@{upstream}", &hash).is_err());
    assert!(store.set_ref("refs/heads/foo.lock", &hash).is_err());

    // Valid name
    assert!(store.set_ref("refs/heads/valid_name", &hash).is_ok());
}
