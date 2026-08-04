mod common;
use common::{alice, encoder, hasher, put_blob, put_tree, setup_refs, setup_store};
use ed25519_dalek::VerifyingKey;
use libvctrl::{
    Command, CreateCommit, EntryKind, LibrageSigner, SetHead, Tree, TreeEntry, VerifyCommit,
};

#[test]
fn test_signed_commit_and_verify() {
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

    let signer = LibrageSigner::generate();
    let vk: VerifyingKey = signer.verifying_key();

    let cmd = CreateCommit {
        tree_hash,
        parents: vec![],
        author: alice(),
        committer: alice(),
        message: "signed".into(),
        encoder: Box::new(encoder()),
        hasher: Box::new(hasher()),
        signer: Some(Box::new(signer)),
    };
    let commit_hash = cmd.execute(&mut store, &mut refs).unwrap();

    let verify = VerifyCommit {
        commit_hash,
        verifying_key: vk,
        encoder: Box::new(encoder()),
        hasher: Box::new(hasher()),
    };
    assert!(verify.execute(&mut store, &mut refs).unwrap());

    let wrong_signer = LibrageSigner::generate();
    let wrong_vk = wrong_signer.verifying_key();
    let verify_wrong = VerifyCommit {
        commit_hash,
        verifying_key: wrong_vk,
        encoder: Box::new(encoder()),
        hasher: Box::new(hasher()),
    };
    assert!(verify_wrong.execute(&mut store, &mut refs).is_err());
}

#[test]
fn test_unsigned_commit() {
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
        message: "unsigned".into(),
        encoder: Box::new(encoder()),
        hasher: Box::new(hasher()),
        signer: None,
    };
    let commit_hash = cmd.execute(&mut store, &mut refs).unwrap();

    let vk = LibrageSigner::generate().verifying_key();
    let verify = VerifyCommit {
        commit_hash,
        verifying_key: vk,
        encoder: Box::new(encoder()),
        hasher: Box::new(hasher()),
    };
    assert!(!verify.execute(&mut store, &mut refs).unwrap());
}
