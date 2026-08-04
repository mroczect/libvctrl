mod common;
use common::{alice, bob, encoder, hasher, put_blob, put_tree, setup_refs, setup_store};
use libvctrl::{
    Command, CreateBranch, CreateCommit, EntryKind, Log, Object, ObjectStore, Revert, SetHead,
    Tree, TreeEntry,
};

#[test]
fn test_revert_with_conflict_added_modified() {
    let mut store = setup_store();
    let mut refs = setup_refs();

    let init_hash = common::blob_hash(b"init");
    CreateBranch {
        name: "refs/heads/main".into(),
        hash: init_hash,
    }
    .execute(&mut store, &mut refs)
    .unwrap();
    SetHead {
        target: "refs/heads/main".into(),
    }
    .execute(&mut store, &mut refs)
    .unwrap();

    let blob1 = put_blob(&mut store, b"data1");
    let entry1 = TreeEntry::new("file.txt".into(), EntryKind::Blob, blob1).unwrap();
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
    let entry2 = TreeEntry::new("file2.txt".into(), EntryKind::Blob, blob2).unwrap();
    let tree2 = Tree::new(vec![
        TreeEntry::new("file.txt".into(), EntryKind::Blob, blob1).unwrap(),
        entry2,
    ])
    .unwrap();
    let tree2_hash = put_tree(&mut store, &tree2);

    let c2 = CreateCommit {
        tree_hash: tree2_hash,
        parents: vec![c1],
        author: alice(),
        committer: alice(),
        message: "add file2".into(),
        encoder: Box::new(encoder()),
        hasher: Box::new(hasher()),
        signer: None,
    }
    .execute(&mut store, &mut refs)
    .unwrap();

    let blob3 = put_blob(&mut store, b"data3");
    let entry2_modified = TreeEntry::new("file2.txt".into(), EntryKind::Blob, blob3).unwrap();
    let tree3 = Tree::new(vec![
        TreeEntry::new("file.txt".into(), EntryKind::Blob, blob1).unwrap(),
        entry2_modified,
    ])
    .unwrap();
    let tree3_hash = put_tree(&mut store, &tree3);

    let c3 = CreateCommit {
        tree_hash: tree3_hash,
        parents: vec![c2],
        author: alice(),
        committer: alice(),
        message: "modify file2".into(),
        encoder: Box::new(encoder()),
        hasher: Box::new(hasher()),
        signer: None,
    }
    .execute(&mut store, &mut refs)
    .unwrap();

    let history_before = Log.execute(&mut store, &mut refs).unwrap();
    assert_eq!(history_before.len(), 3);
    assert_eq!(history_before[0].message, "modify file2");
    assert_eq!(history_before[0].tree, tree3_hash);

    let c3_obj = store.get(&c3).unwrap().unwrap();
    assert!(matches!(c3_obj, Object::Commit(_)));

    let revert = Revert {
        commit_hash: c2,
        author: bob(),
        committer: bob(),
        encoder: Box::new(encoder()),
        hasher: Box::new(hasher()),
    };
    let err = revert.execute(&mut store, &mut refs).unwrap_err();
    assert!(err.to_string().contains("has been modified"));
}
