mod common;
use common::{alice, bob, encoder, hasher, put_blob, put_tree, setup_refs, setup_store};
use libvctrl::{
    CherryPick, Command, ConflictResolver, CreateCommit, EntryKind, Log, Object, ObjectStore,
    SetHead, ThreeWayMerger, Tree, TreeEntry,
};

struct NoConflictResolver;
impl ConflictResolver for NoConflictResolver {
    fn resolve(&self, _: &[u8], _: &[u8], _: &[u8]) -> Option<Vec<u8>> {
        None
    }
}

#[test]
fn test_cherry_pick_simple() {
    let mut store = setup_store();
    let mut refs = setup_refs();

    let set_head = SetHead {
        target: "refs/heads/main".into(),
    };
    set_head.execute(&mut store, &mut refs).unwrap();

    let blob1 = put_blob(&mut store, b"data1");
    let entry1 = TreeEntry::new("file.txt".into(), EntryKind::Blob, blob1);
    let tree1 = Tree::new(vec![entry1]).unwrap();
    let tree1_hash = put_tree(&mut store, &tree1);

    let c1 = CreateCommit {
        tree_hash: tree1_hash,
        parents: vec![],
        author: alice(),
        committer: alice(),
        message: "first".into(),
        encoder: Box::new(encoder()),
        hasher: Box::new(hasher()),
        signer: None,
    }
    .execute(&mut store, &mut refs)
    .unwrap();

    let blob2 = put_blob(&mut store, b"data2");
    let entry2 = TreeEntry::new("file2.txt".into(), EntryKind::Blob, blob2);
    let tree2 = Tree::new(vec![
        TreeEntry::new("file.txt".into(), EntryKind::Blob, blob1),
        entry2,
    ])
    .unwrap();
    let tree2_hash = put_tree(&mut store, &tree2);

    let c2 = CreateCommit {
        tree_hash: tree2_hash,
        parents: vec![c1],
        author: alice(),
        committer: alice(),
        message: "second".into(),
        encoder: Box::new(encoder()),
        hasher: Box::new(hasher()),
        signer: None,
    }
    .execute(&mut store, &mut refs)
    .unwrap();

    libvctrl::CreateBranch {
        name: "refs/heads/other".into(),
        hash: c1,
    }
    .execute(&mut store, &mut refs)
    .unwrap();

    SetHead {
        target: "refs/heads/other".into(),
    }
    .execute(&mut store, &mut refs)
    .unwrap();

    let cherry = CherryPick {
        commit_hash: c2,
        author: bob(),
        committer: bob(),
        merger: Box::new(ThreeWayMerger),
        resolver: Box::new(NoConflictResolver),
        encoder: Box::new(encoder()),
        hasher: Box::new(hasher()),
    };
    let cherry_hash = cherry.execute(&mut store, &mut refs).unwrap();

    let log = Log.execute(&mut store, &mut refs).unwrap();
    assert_eq!(log.len(), 2);
    assert!(log[0].message.contains("cherry-pick"));

    let cherry_commit = match store.get(&cherry_hash).unwrap().unwrap() {
        Object::Commit(c) => c,
        _ => panic!(),
    };
    let tree = match store.get(&cherry_commit.tree).unwrap().unwrap() {
        Object::Tree(t) => t,
        _ => panic!(),
    };
    assert_eq!(tree.entries().len(), 2);
}

#[test]
fn test_revert() {
    let mut store = setup_store();
    let mut refs = setup_refs();

    let set_head = SetHead {
        target: "refs/heads/main".into(),
    };
    set_head.execute(&mut store, &mut refs).unwrap();

    let blob1 = put_blob(&mut store, b"data1");
    let entry1 = TreeEntry::new("file.txt".into(), EntryKind::Blob, blob1);
    let tree1 = Tree::new(vec![entry1]).unwrap();
    let tree1_hash = put_tree(&mut store, &tree1);

    let c1 = CreateCommit {
        tree_hash: tree1_hash,
        parents: vec![],
        author: alice(),
        committer: alice(),
        message: "first".into(),
        encoder: Box::new(encoder()),
        hasher: Box::new(hasher()),
        signer: None,
    }
    .execute(&mut store, &mut refs)
    .unwrap();

    let blob2 = put_blob(&mut store, b"data2");
    let entry2 = TreeEntry::new("file2.txt".into(), EntryKind::Blob, blob2);
    let tree2 = Tree::new(vec![
        TreeEntry::new("file.txt".into(), EntryKind::Blob, blob1),
        entry2,
    ])
    .unwrap();
    let tree2_hash = put_tree(&mut store, &tree2);

    let c2 = CreateCommit {
        tree_hash: tree2_hash,
        parents: vec![c1],
        author: alice(),
        committer: alice(),
        message: "second".into(),
        encoder: Box::new(encoder()),
        hasher: Box::new(hasher()),
        signer: None,
    }
    .execute(&mut store, &mut refs)
    .unwrap();

    let revert = libvctrl::Revert {
        commit_hash: c2,
        author: bob(),
        committer: bob(),
        encoder: Box::new(encoder()),
        hasher: Box::new(hasher()),
    };
    let revert_hash = revert.execute(&mut store, &mut refs).unwrap();

    let log = Log.execute(&mut store, &mut refs).unwrap();
    assert_eq!(log.len(), 3);
    assert!(log[0].message.contains("Revert"));

    let revert_commit = match store.get(&revert_hash).unwrap().unwrap() {
        Object::Commit(c) => c,
        _ => panic!(),
    };
    let tree = match store.get(&revert_commit.tree).unwrap().unwrap() {
        Object::Tree(t) => t,
        _ => panic!(),
    };
    assert_eq!(tree.entries().len(), 1);
    assert_eq!(tree.entries()[0].name, "file.txt");
}
