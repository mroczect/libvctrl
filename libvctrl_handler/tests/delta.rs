use criterion as _;
use libvctrl_handler::{
    ChangeKind, Conflict, FileDelta, HASH_LENGTH, Hash, MergeResult, TreeDelta,
};
use std::path::{Path, PathBuf};

mod common;

fn h(byte: u8) -> Hash {
    Hash::from([byte; HASH_LENGTH])
}

#[test]
fn test_file_delta_added() {
    let h1 = h(1);
    let delta = FileDelta::added(PathBuf::from("a.txt"), h1);

    assert!(delta.is_added());
    assert!(!delta.is_deleted());
    assert!(!delta.is_modified());
    assert!(!delta.is_type_change());
    assert!(!delta.is_renamed());
    assert!(!delta.is_copied());

    assert_eq!(delta.path(), Path::new("a.txt"));
    assert_eq!(delta.old_path(), None);
    assert_eq!(delta.old_hash(), None);
    assert_eq!(delta.new_hash(), Some(h1));
    assert_eq!(delta.kind(), ChangeKind::Added);
}

#[test]
fn test_file_delta_deleted() {
    let h1 = h(1);
    let delta = FileDelta::deleted(PathBuf::from("a.txt"), h1);

    assert!(delta.is_deleted());
    assert!(!delta.is_added());
    assert_eq!(delta.path(), Path::new("a.txt"));
    assert_eq!(delta.old_hash(), Some(h1));
    assert_eq!(delta.new_hash(), None);
    assert_eq!(delta.kind(), ChangeKind::Deleted);
}

#[test]
fn test_file_delta_modified_and_type_change() {
    let h1 = h(1);
    let h2 = h(2);

    let modified = FileDelta::modified(PathBuf::from("a.txt"), h1, h2);
    assert!(modified.is_modified());
    assert_eq!(modified.old_hash(), Some(h1));
    assert_eq!(modified.new_hash(), Some(h2));
    assert_eq!(modified.kind(), ChangeKind::Modified);

    let type_change = FileDelta::type_change(PathBuf::from("a.txt"), h1, h2);
    assert!(type_change.is_type_change());
    assert_eq!(type_change.old_hash(), Some(h1));
    assert_eq!(type_change.new_hash(), Some(h2));
    assert_eq!(type_change.kind(), ChangeKind::TypeChange);
}

#[test]
fn test_file_delta_renamed_and_copied() {
    let h1 = h(1);
    let h2 = h(2);

    let renamed = FileDelta::renamed(PathBuf::from("old.txt"), PathBuf::from("new.txt"), h1, h2);
    assert!(renamed.is_renamed());
    assert_eq!(renamed.path(), Path::new("new.txt"));
    assert_eq!(renamed.old_path(), Some(Path::new("old.txt")));
    assert_eq!(renamed.old_hash(), Some(h1));
    assert_eq!(renamed.new_hash(), Some(h2));
    assert_eq!(renamed.kind(), ChangeKind::Renamed);

    let copied = FileDelta::copied(PathBuf::from("old.txt"), PathBuf::from("copy.txt"), h1, h2);
    assert!(copied.is_copied());
    assert_eq!(copied.path(), Path::new("copy.txt"));
    assert_eq!(copied.old_path(), Some(Path::new("old.txt")));
    assert_eq!(copied.kind(), ChangeKind::Copied);
}

#[test]
fn test_tree_delta_basic() {
    let delta = TreeDelta::new();
    assert!(delta.is_empty());
    assert_eq!(delta.len(), 0);
    assert_eq!(delta.changes().len(), 0);
    assert_eq!(delta.iter().count(), 0);
}

#[test]
fn test_tree_delta_from_changes() {
    let h1 = h(1);
    let changes = vec![
        FileDelta::added(PathBuf::from("a.txt"), h1),
        FileDelta::added(PathBuf::from("b.txt"), h1),
    ];

    let delta = TreeDelta::from_changes(changes);
    assert!(!delta.is_empty());
    assert_eq!(delta.len(), 2);
    assert_eq!(delta.changes().len(), 2);
    assert_eq!(delta.iter().count(), 2);
    assert_eq!(delta.into_iter().count(), 2);
}

#[test]
fn test_tree_delta_iter_by_ref() {
    let h1 = h(1);
    let delta = TreeDelta::from_changes(vec![FileDelta::added(PathBuf::from("a.txt"), h1)]);

    let refs: Vec<&FileDelta> = (&delta).into_iter().collect();
    assert_eq!(refs.len(), 1);
    assert_eq!(
        refs.first().map(|delta| delta.path()),
        Some(Path::new("a.txt"))
    );
}

#[test]
fn test_conflict_accessors() {
    let ancestor = h(1);
    let ours = h(2);
    let theirs = h(3);

    let conflict = Conflict::new(PathBuf::from("file.txt"), ancestor, ours, theirs);

    assert_eq!(conflict.path(), Path::new("file.txt"));
    assert_eq!(conflict.ancestor_blob(), ancestor);
    assert_eq!(conflict.our_blob(), ours);
    assert_eq!(conflict.their_blob(), theirs);
}

#[test]
fn test_merge_result_variants() {
    let h1 = h(1);
    let success = MergeResult::Success(h1);
    assert!(success.is_success());
    assert!(!success.is_conflicts());
    assert!(success.conflicts().is_none());

    let conflict = Conflict::new(PathBuf::from("file.txt"), h1, h(2), h(3));
    let conflicts = MergeResult::Conflicts(vec![conflict]);
    assert!(!conflicts.is_success());
    assert!(conflicts.is_conflicts());

    let conflict_list = conflicts.conflicts();
    assert!(conflict_list.is_some(), "expected conflicts");
    if let Some(c) = conflict_list {
        assert_eq!(c.len(), 1);
    } else {
        loop {
            core::hint::spin_loop();
        }
    }
}
