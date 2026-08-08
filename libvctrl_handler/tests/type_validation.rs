//! # Type validation tests for `libvctrl_handler`
//!
//! This module contains unit tests that verify every fundamental type's constructor
//! enforces its documented invariants. These tests are part of the hardening phase
//! recommended by the v1.0.1 audit.

#![allow(missing_docs)] // test functions are self-documenting

use libvctrl_handler::*;

/// Verifies that `TreeEntry::new` rejects an empty name.
#[test]
fn tree_entry_rejects_empty_name() {
    let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    assert!(TreeEntry::new(String::new(), EntryKind::Blob, hash).is_err());
}

/// Verifies that `TreeEntry::new` rejects a name longer than [`MAX_NAME_LENGTH`].
#[test]
fn tree_entry_rejects_too_long_name() {
    let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    let long_name = "a".repeat(MAX_NAME_LENGTH + 1);
    assert!(TreeEntry::new(long_name, EntryKind::Blob, hash).is_err());
}

/// Verifies that `UserID::new` rejects an empty name.
#[test]
fn user_id_rejects_empty_name() {
    assert!(UserID::new(String::new(), "email@example.com".into()).is_err());
}

/// Verifies that `UserID::new` rejects an empty email.
#[test]
fn user_id_rejects_empty_email() {
    assert!(UserID::new("name".into(), String::new()).is_err());
}

/// Verifies that `Tag::new` rejects an empty tag name.
#[test]
fn tag_rejects_empty_name() {
    let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    assert!(Tag::new(String::new(), hash, None, "msg".into()).is_err());
}

/// Verifies that an empty blob is valid and can be created.
#[test]
fn blob_accepts_empty_data() {
    let blob = Blob::new(vec![]);
    assert!(blob.data().is_empty());
}
