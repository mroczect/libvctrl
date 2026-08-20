use criterion as _;
use libvctrl_handler::{
    EntryKind, HASH_LENGTH, Hash, MAX_TREE_ENTRIES, Tree, TreeEntry, VctrlError,
};
mod common;

fn h() -> Hash {
    Hash::from([0_u8; HASH_LENGTH])
}

#[test]
fn test_tree_entry_valid() {
    let hash = h();
    let entry = common::ok(TreeEntry::new(
        "file.txt".to_string(),
        EntryKind::Blob,
        hash,
    ));

    assert_eq!(entry.name(), "file.txt");
    assert_eq!(entry.kind(), EntryKind::Blob);
    assert_eq!(entry.hash(), &hash);
}

#[test]
fn test_tree_entry_invalid_name() {
    let result = TreeEntry::new("a/b".to_string(), EntryKind::Blob, h());
    assert!(result.is_err());
}

#[test]
fn test_tree_new_empty() {
    let tree = common::ok(Tree::new(Vec::new()));
    assert!(tree.is_empty());
    assert_eq!(tree.len(), 0);
    assert_eq!(tree.entries().len(), 0);
}

#[test]
fn test_tree_new_sorts_entries() {
    let e1 = common::ok(TreeEntry::new("b".to_string(), EntryKind::Blob, h()));
    let e2 = common::ok(TreeEntry::new("a".to_string(), EntryKind::Blob, h()));

    let tree = common::ok(Tree::new(vec![e1, e2]));

    assert_eq!(tree.len(), 2);
    assert_eq!(tree.entries().first().map(TreeEntry::name), Some("a"));
    assert_eq!(tree.entries().get(1).map(TreeEntry::name), Some("b"));
}

#[test]
fn test_tree_new_duplicate_name() {
    let dup1 = common::ok(TreeEntry::new("x".to_string(), EntryKind::Blob, h()));
    let dup2 = common::ok(TreeEntry::new("x".to_string(), EntryKind::Tree, h()));

    let result = Tree::new(vec![dup1, dup2]);
    assert!(result.is_err());
    assert_eq!(
        common::err(result),
        VctrlError::InvalidTreeStructure("duplicate entry name: 'x'".to_string())
    );
}

#[test]
fn test_tree_new_exceeds_max_entries() {
    let max_entries = usize::try_from(MAX_TREE_ENTRIES).unwrap_or(usize::MAX);
    let entries = (0..=max_entries)
        .map(|i| common::ok(TreeEntry::new(format!("entry{i}"), EntryKind::Blob, h())))
        .collect::<Vec<_>>();

    let result = Tree::new(entries);
    assert!(result.is_err());

    let err = common::err(result);
    assert!(
        matches!(&err, VctrlError::ExceededMaxSize(_)),
        "unexpected error: {err:?}"
    );
}

#[test]
fn test_tree_get() {
    let e = common::ok(TreeEntry::new("a".to_string(), EntryKind::Blob, h()));
    let tree = common::ok(Tree::new(vec![e]));

    assert_eq!(tree.get("a").map(TreeEntry::name), Some("a"));
    assert!(tree.get("missing").is_none());
}
