use criterion as _;
use libvctrl_handler::{
    Commit, CommitMeta, HASH_LENGTH, Hash, MAX_MESSAGE_LENGTH, MAX_PARENT_COUNT, UserID, VctrlError,
};
mod common;

fn h(byte: u8) -> Hash {
    Hash::from([byte; HASH_LENGTH])
}

fn user() -> UserID {
    common::ok(UserID::new(
        "Alice".to_string(),
        "alice@example.com".to_string(),
    ))
}

#[test]
fn test_commit_new_valid_empty_parents() {
    let tree = h(1);
    let author = user();
    let committer = user();

    let commit = common::ok(Commit::new(
        tree,
        Vec::new(),
        author.clone(),
        committer.clone(),
        "initial commit".to_string(),
    ));

    assert_eq!(commit.tree(), &tree);
    assert!(commit.parents().is_empty());
    assert_eq!(commit.author(), &author);
    assert_eq!(commit.committer(), &committer);
    assert_eq!(commit.message(), "initial commit");
    assert_eq!(commit.meta().timestamp(), 0);
    assert_eq!(commit.meta().timezone_offset(), 0);
}

#[test]
fn test_commit_new_duplicate_parent() {
    let tree = h(1);
    let parent = h(2);
    let author = user();
    let committer = user();

    let result = Commit::new(
        tree,
        vec![parent, parent],
        author,
        committer,
        "duplicate".to_string(),
    );

    assert!(result.is_err());
    assert_eq!(common::err(result), VctrlError::DuplicateParent);
}

#[test]
fn test_commit_new_too_many_parents() {
    let tree = h(1);
    let parent = h(2);
    let author = user();
    let committer = user();

    let max_parents = usize::try_from(MAX_PARENT_COUNT).unwrap_or(usize::MAX);
    let parents = vec![parent; max_parents + 1];

    let result = Commit::new(tree, parents, author, committer, "many parents".to_string());

    assert!(result.is_err());
    let err = common::err(result);
    assert!(
        matches!(&err, VctrlError::ExceededMaxSize(_)),
        "unexpected error: {err:?}"
    );
}

#[test]
fn test_commit_new_message_too_long() {
    let tree = h(1);
    let author = user();
    let committer = user();
    let max_msg = usize::try_from(MAX_MESSAGE_LENGTH).unwrap_or(usize::MAX);
    let message = "a".repeat(max_msg + 1);

    let result = Commit::new(tree, Vec::new(), author, committer, message);

    assert!(result.is_err());
    let err = common::err(result);
    assert!(
        matches!(&err, VctrlError::ExceededMaxSize(_)),
        "unexpected error: {err:?}"
    );
}

#[test]
fn test_commit_with_meta() {
    let tree = h(1);
    let author = user();
    let committer = user();
    let meta = common::ok(CommitMeta::new(1_700_000_000, 120, Some("utf-8".into())));

    let commit = common::ok(Commit::with_meta(
        tree,
        Vec::new(),
        author,
        committer,
        "meta commit".to_string(),
        meta,
    ));

    assert_eq!(commit.meta().timestamp(), 1_700_000_000);
    assert_eq!(commit.meta().timezone_offset(), 120);
    assert_eq!(commit.meta().encoding(), Some("utf-8"));
}
