#![allow(missing_docs)]

use libvctrl_handler::*;

#[test]
fn tree_entry_rejects_empty_name() {
    let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    assert!(TreeEntry::new(String::new(), EntryKind::Blob, hash).is_err());
}

#[test]
fn tree_entry_rejects_too_long_name() {
    let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    let long_name = "a".repeat(MAX_NAME_LENGTH + 1);
    assert!(TreeEntry::new(long_name, EntryKind::Blob, hash).is_err());
}

#[test]
fn user_id_rejects_empty_name() {
    assert!(UserID::new(String::new(), "email@example.com".into()).is_err());
}

#[test]
fn user_id_rejects_empty_email() {
    assert!(UserID::new("name".into(), String::new()).is_err());
}

#[test]
fn tag_rejects_empty_name() {
    let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    assert!(Tag::new(String::new(), hash, None, "msg".into()).is_err());
}

#[test]
fn blob_accepts_empty_data() {
    let blob = Blob::new(vec![]);
    assert!(blob.data().is_empty());
}
