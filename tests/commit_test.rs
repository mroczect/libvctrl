mod common;
use common::{alice, bob, setup_refs, setup_store};

use libvctrl::{Blob, Commit, Object, ObjectStore, RefStore, Tree, create_commit, get_commit, log};

#[test]
fn commit_new_via_create_commit() {
    let mut store = setup_store();
    let mut refs = setup_refs();

    let tree = Tree::new(vec![]).unwrap();
    let tree_hash = store.put(&Object::Tree(tree)).unwrap();

    let c1 = create_commit(
        &mut store,
        tree_hash,
        vec![],
        alice(),
        alice(),
        "init".into(),
    )
    .unwrap();
    refs.set_ref("refs/heads/main", &c1).unwrap();
    refs.set_head("refs/heads/main").unwrap();

    let commit = get_commit(&store, &c1).unwrap().unwrap();
    assert_eq!(commit.message(), "init");
    assert_eq!(commit.parents().len(), 0);
    assert_eq!(commit.tree(), &tree_hash);

    let history = log(&store, &refs).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].message(), "init");
}

#[test]
fn commit_chain_log() {
    let mut store = setup_store();
    let mut refs = setup_refs();

    let tree = Tree::new(vec![]).unwrap();
    let tree_hash = store.put(&Object::Tree(tree)).unwrap();

    let c1 = create_commit(
        &mut store,
        tree_hash,
        vec![],
        alice(),
        alice(),
        "first".into(),
    )
    .unwrap();
    refs.set_ref("refs/heads/main", &c1).unwrap();
    refs.set_head("refs/heads/main").unwrap();

    let c2 = create_commit(
        &mut store,
        tree_hash,
        vec![c1],
        bob(),
        bob(),
        "second".into(),
    )
    .unwrap();
    refs.set_ref("refs/heads/main", &c2).unwrap();

    let history = log(&store, &refs).unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].message(), "second");
    assert_eq!(history[1].message(), "first");
}

#[test]
fn commit_new_validates_hash() {
    let tree_hash = Blob::new(b"t".to_vec()).hash().unwrap();
    let commit = Commit::new(tree_hash, vec![], alice(), alice(), "msg".into(), None);
    assert!(commit.is_ok());
}

#[test]
fn commit_getters() {
    let tree_hash = Blob::new(b"t".to_vec()).hash().unwrap();
    let author = alice();
    let committer = bob();
    let commit = Commit::new(
        tree_hash,
        vec![],
        author.clone(),
        committer.clone(),
        "getter test".into(),
        Some(vec![1, 2, 3]),
    )
    .unwrap();

    assert_eq!(commit.tree(), &tree_hash);
    assert_eq!(commit.message(), "getter test");
    assert_eq!(commit.author().name, author.name);
    assert_eq!(commit.committer().email, committer.email);
    assert_eq!(commit.signature(), Some(&[1, 2, 3][..]));
    assert!(commit.timestamp().timestamp() > 0);
}
