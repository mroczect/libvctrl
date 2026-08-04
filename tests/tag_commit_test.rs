mod common;
use common::{alice, encoder, hasher, put_blob, put_tree, setup_refs, setup_store};
use libvctrl::{
    Command, CreateCommit, CreateLightweightTag, EntryKind, Log, RefStore, SetHead, Tree, TreeEntry,
};

#[test]
fn test_commit_on_tag_detached_head() {
    let mut store = setup_store();
    let mut refs = setup_refs();

    let blob_hash = put_blob(&mut store, b"data");
    let entry = TreeEntry::new("file.txt".into(), EntryKind::Blob, blob_hash);
    let tree = Tree::new(vec![entry]).unwrap();
    let tree_hash = put_tree(&mut store, &tree);

    let c1 = CreateCommit {
        tree_hash,
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

    CreateLightweightTag {
        name: "v1.0".into(),
        target: c1,
    }
    .execute(&mut store, &mut refs)
    .unwrap();

    SetHead {
        target: c1.to_hex(),
    }
    .execute(&mut store, &mut refs)
    .unwrap();

    let c2 = CreateCommit {
        tree_hash,
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

    let c2_commit = libvctrl::ObjectStore::get(&store, &c2)
        .unwrap()
        .expect("c2 harus tersimpan");
    assert!(matches!(c2_commit, libvctrl::Object::Commit(_)));

    let tag_ref = refs.get_ref("refs/tags/v1.0").unwrap().unwrap();
    assert_eq!(tag_ref, c1, "tag should not move");

    let history = Log.execute(&mut store, &mut refs).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].message, "first");
}
