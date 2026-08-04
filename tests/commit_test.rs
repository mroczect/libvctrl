mod common;
use common::{alice, bob, put_blob, put_tree, setup_refs, setup_store};

use libvctrl::{Command, CreateCommit, EntryKind, Log, SetHead, Tree, TreeEntry};

#[test]
fn create_commit_and_log() {
    let mut store = setup_store();
    let mut refs = setup_refs();

    let set_head = SetHead {
        target: "refs/heads/main".into(),
    };
    set_head.execute(&mut store, &mut refs).unwrap();

    let blob_hash = put_blob(&mut store, b"data");
    let entry = TreeEntry::new("file.txt".into(), EntryKind::Blob, blob_hash);
    let tree = Tree::new(vec![entry]).unwrap();
    let tree_hash = put_tree(&mut store, &tree);

    let cmd = CreateCommit {
        tree_hash,
        parents: vec![],
        author: alice(),
        committer: alice(),
        message: "initial".into(),
        encoder: Box::new(common::encoder()),
        hasher: Box::new(common::hasher()),
    };
    let commit_hash = cmd.execute(&mut store, &mut refs).unwrap();
    assert_eq!(commit_hash.as_bytes().len(), 64);

    let log_cmd = Log;
    let history = log_cmd.execute(&mut store, &mut refs).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].message, "initial");
    assert_eq!(history[0].tree, tree_hash);
    assert_eq!(history[0].parents.len(), 0);
}

#[test]
fn commit_chain_log() {
    let mut store = setup_store();
    let mut refs = setup_refs();

    let set_head = SetHead {
        target: "refs/heads/main".into(),
    };
    set_head.execute(&mut store, &mut refs).unwrap();

    let blob_hash = put_blob(&mut store, b"data");
    let entry = TreeEntry::new("f".into(), EntryKind::Blob, blob_hash);
    let tree = Tree::new(vec![entry]).unwrap();
    let tree_hash = put_tree(&mut store, &tree);

    let cmd1 = CreateCommit {
        tree_hash,
        parents: vec![],
        author: alice(),
        committer: alice(),
        message: "first".into(),
        encoder: Box::new(common::encoder()),
        hasher: Box::new(common::hasher()),
    };
    let c1 = cmd1.execute(&mut store, &mut refs).unwrap();

    let cmd2 = CreateCommit {
        tree_hash,
        parents: vec![c1],
        author: bob(),
        committer: bob(),
        message: "second".into(),
        encoder: Box::new(common::encoder()),
        hasher: Box::new(common::hasher()),
    };
    let _c2 = cmd2.execute(&mut store, &mut refs).unwrap();

    let log_cmd = Log;
    let history = log_cmd.execute(&mut store, &mut refs).unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].message, "second");
    assert_eq!(history[1].message, "first");
}

#[test]
fn commit_getters() {
    let blob_hash = put_blob(&mut setup_store(), b"t");
    let author = alice();
    let committer = bob();
    let commit = libvctrl::Commit::new(
        blob_hash,
        vec![],
        author.clone(),
        committer.clone(),
        "getter test".into(),
        Some(vec![1, 2, 3]),
    );
    assert_eq!(commit.tree, blob_hash);
    assert_eq!(commit.message, "getter test");
    assert_eq!(commit.author.name, author.name);
    assert_eq!(commit.committer.email, committer.email);
    assert_eq!(commit.signature, Some(vec![1, 2, 3]));
    assert!(commit.timestamp.timestamp() > 0);
}
