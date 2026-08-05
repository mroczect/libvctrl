mod common;
use common::*;
use libvctrl::*;

#[test]
fn blob_roundtrip() {
    let data = vec![1, 2, 3];
    let blob = Blob::new(data.clone());
    assert_eq!(blob.as_bytes(), &data[..]);
    assert_eq!(blob.into_bytes(), data);
}

#[test]
fn tree_new_empty() {
    let tree = Tree::new(vec![]).unwrap();
    assert!(tree.is_empty());
}

#[test]
fn tree_duplicate_name() {
    let h = blob_hash(b"x");
    let e1 = TreeEntry::new("a".into(), EntryKind::Blob, h).unwrap();
    let e2 = TreeEntry::new("a".into(), EntryKind::Blob, h).unwrap();
    assert!(Tree::new(vec![e1, e2]).is_err());
}

#[test]
fn tree_entry_invalid_name() {
    let h = blob_hash(b"x");
    assert!(TreeEntry::new("".into(), EntryKind::Blob, h).is_err());
    assert!(TreeEntry::new("a/b".into(), EntryKind::Blob, h).is_err());
    assert!(TreeEntry::new("..".into(), EntryKind::Blob, h).is_err());
    assert!(TreeEntry::new(".".into(), EntryKind::Blob, h).is_err());
}

#[test]
fn hash_hex_conversion() {
    let h = blob_hash(b"data");
    let hex = h.to_hex();
    let h2 = Hash::from_hex(&hex).unwrap();
    assert_eq!(h, h2);
    assert!(Hash::from_hex("short").is_err());
}

#[test]
fn user_id_validation() {
    assert!(UserID::new("".into(), "a@b.com".into()).is_err());
    assert!(UserID::new("a".into(), "".into()).is_err());
    let u = UserID::new("Alice".into(), "a@b.com".into()).unwrap();
    assert_eq!(u.name, "Alice");
    assert_eq!(u.email, "a@b.com");
}

#[test]
fn tag_creation_and_signature() {
    let target = blob_hash(b"target");
    let tagger = alice();
    let mut tag = Tag::new(target, tagger.clone(), "msg".into());
    assert_eq!(tag.target, target);
    assert_eq!(tag.tagger, tagger);
    assert_eq!(tag.message, "msg");
    assert!(tag.signature.is_none());
    tag.signature = Some(vec![1, 2, 3]);
    assert_eq!(tag.signature, Some(vec![1, 2, 3]));
}
