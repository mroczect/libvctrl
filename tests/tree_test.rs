mod common;
use common::blob_hash;

use libvctrl::{EntryKind, Tree, TreeEntry, TreeError};

fn make_entry(name: &str, hash: libvctrl::Hash) -> TreeEntry {
    TreeEntry::new(name.to_string(), EntryKind::Blob, hash).unwrap()
}

#[test]
fn tree_new_sorts_entries() {
    let h = blob_hash(b"x");
    let entries = vec![make_entry("c", h), make_entry("a", h), make_entry("b", h)];
    let tree = Tree::new(entries).unwrap();
    let names: Vec<&str> = tree.entries().iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, vec!["a", "b", "c"]);
}

#[test]
fn tree_duplicate_entries_error() {
    let h = blob_hash(b"x");
    let entries = vec![make_entry("a", h), make_entry("a", h)];
    let err = Tree::new(entries).unwrap_err();
    match err {
        TreeError::DuplicateEntry(name) => assert_eq!(name, "a"),
        _ => panic!("expected DuplicateEntry"),
    }
}

#[test]
fn tree_hash_deterministic() {
    let h = blob_hash(b"d");
    let entries1 = vec![make_entry("a", h), make_entry("b", h)];
    let entries2 = vec![make_entry("b", h), make_entry("a", h)];
    let tree1 = Tree::new(entries1).unwrap();
    let tree2 = Tree::new(entries2).unwrap();
    let hash1 = common::tree_hash(&tree1);
    let hash2 = common::tree_hash(&tree2);
    assert_eq!(hash1, hash2);
}

#[test]
fn tree_empty() {
    let tree = Tree::new(vec![]).unwrap();
    assert!(tree.is_empty());
    assert_eq!(tree.entries().len(), 0);
}

#[test]
fn tree_into_entries() {
    let h = blob_hash(b"e");
    let entries = vec![make_entry("x", h)];
    let tree = Tree::new(entries.clone()).unwrap();
    assert_eq!(tree.into_entries().len(), 1);
}
