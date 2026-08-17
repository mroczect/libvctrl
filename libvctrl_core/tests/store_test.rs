//! # Store and RefStore Integration Tests
//!
//! This module contains integration-style tests for the in-memory object and
//! reference store implementations:
//!
//! - `MemoryStore` implements `ObjectStore` and provides CRUD operations plus
//!   streaming reads via `Box<dyn Read>`.
//! - `MemoryRefStore` implements `RefStore` and manages named references with
//!   strict name validation and deterministic sorted iteration.
//!
//! The tests verify both normal behavior and defensive handling of malformed
//! or potentially hostile inputs.

use libvctrl_core::hash::Sha512Hasher;
use libvctrl_core::store::{MemoryRefStore, MemoryStore};
use libvctrl_handler::{Hash, Hasher, MAX_NAME_LENGTH, ObjectStore, RefStore};
use std::io::Read;

/// Computes a SHA-512 content hash for the given data.
///
/// This helper uses `Sha512Hasher` to derive a stable, content-addressed
/// identifier. It is used to generate distinct `Hash` values for objects and
/// references in the tests.
fn dummy_hash_from_data(data: &[u8]) -> Hash {
    let hasher = Sha512Hasher;
    hasher.hash(data).unwrap()
}

/// Tests CRUD operations and streaming reads for `MemoryStore`.
///
/// Verifies:
/// - `put` stores data and `exists` reports it correctly.
/// - `get` returns a stream that yields the exact stored bytes.
/// - `delete` removes the object and subsequent `get` fails.
/// - Deleting or reading a non-existent object does not panic.
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

/// Tests that `MemoryStore` can stream a large object without requiring a
/// full contiguous copy beyond the stored data.
///
/// The object is 10 MiB; reading it back through the returned reader must
/// yield the exact original bytes.
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

/// Tests CRUD operations and sorted iteration for `MemoryRefStore`.
///
/// Verifies:
/// - References can be set and retrieved.
/// - `list_refs` returns names in sorted order.
/// - Deleting a reference removes it from the store and from the listing.
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

/// Tests that `MemoryRefStore` enforces strict reference name validation.
///
/// The following invalid names are rejected:
/// - Empty string.
/// - Names exceeding `MAX_NAME_LENGTH`.
/// - Path traversal attempts (`../`, `..\\`, `..`).
/// - Git illegal characters (`~`, `^`, `:`, space, `@{`, ending with `.lock`).
///
/// A normal valid name is accepted.
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
