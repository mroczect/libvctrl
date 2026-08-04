use libvctrl::{Blob, EntryKind, Hash, Tree, TreeEntry, TreeError};

fn make_entry(name: &str, hash: Hash) -> TreeEntry {
    TreeEntry::new(name.to_string(), EntryKind::Blob, hash)
}

#[test]
fn tree_new_sorts_entries() {
    let h = Blob::new(b"x".to_vec()).hash().unwrap();
    let entries = vec![make_entry("c", h), make_entry("a", h), make_entry("b", h)];
    let tree = Tree::new(entries).unwrap();
    let names: Vec<&str> = tree.entries().iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, vec!["a", "b", "c"]);
}

#[test]
fn tree_duplicate_entries_error() {
    let h = Blob::new(b"x".to_vec()).hash().unwrap();
    let entries = vec![make_entry("a", h), make_entry("a", h)];
    let err = Tree::new(entries).unwrap_err();
    match err {
        TreeError::DuplicateEntry(name) => assert_eq!(name, "a"),
    }
}

#[test]
fn tree_hash_deterministic() {
    let h = Blob::new(b"d".to_vec()).hash().unwrap();
    let entries1 = vec![make_entry("a", h), make_entry("b", h)];
    let entries2 = vec![make_entry("b", h), make_entry("a", h)];
    let tree1 = Tree::new(entries1).unwrap();
    let tree2 = Tree::new(entries2).unwrap();
    assert_eq!(tree1.hash().unwrap(), tree2.hash().unwrap());
}

#[test]
fn tree_empty() {
    let tree = Tree::new(vec![]).unwrap();
    assert!(tree.is_empty());
    assert_eq!(tree.entries().len(), 0);
    assert!(tree.hash().is_ok());
}

#[test]
fn tree_into_entries() {
    let h = Blob::new(b"e".to_vec()).hash().unwrap();
    let entries = vec![make_entry("x", h)];
    let tree = Tree::new(entries.clone()).unwrap();
    assert_eq!(tree.into_entries().len(), 1);
}
