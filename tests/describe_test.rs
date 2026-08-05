mod common;
use common::*;
use libvctrl::*;

#[test]
fn describe_finds_closest_tag() {
    let mut store = MemoryStore::new();
    let mut refs = MemoryRefStore::new();

    let tree_hash = put_tree(&mut store, &Tree::new(vec![]).unwrap());
    let c1 = Commit::new(tree_hash, vec![], alice(), alice(), "initial".into(), None);
    let h1 = commit_hash(&c1);
    store.put(&h1, &Object::Commit(Box::new(c1))).unwrap();

    refs.set_ref("refs/tags/v1.0", &h1).unwrap();

    let c2 = Commit::new(tree_hash, vec![h1], alice(), alice(), "second".into(), None);
    let h2 = commit_hash(&c2);
    store.put(&h2, &Object::Commit(Box::new(c2))).unwrap();

    let desc = Describe {
        commit_hash: h2,
        max_commits_to_search: 10,
    }
    .execute(&mut store, &mut refs)
    .unwrap()
    .unwrap();
    assert!(desc.starts_with("v1.0-1-g"));
}

#[test]
fn describe_no_tags() {
    let mut store = MemoryStore::new();
    let mut refs = MemoryRefStore::new();

    let tree_hash = put_tree(&mut store, &Tree::new(vec![]).unwrap());
    let commit = Commit::new(tree_hash, vec![], alice(), alice(), "untagged".into(), None);
    let h = commit_hash(&commit);
    store.put(&h, &Object::Commit(Box::new(commit))).unwrap();

    let desc = Describe {
        commit_hash: h,
        max_commits_to_search: 10,
    }
    .execute(&mut store, &mut refs)
    .unwrap();
    assert!(desc.is_none());
}

#[test]
fn describe_commit_not_found() {
    let mut store = MemoryStore::new();
    let mut refs = MemoryRefStore::new();
    let fake = blob_hash(b"nonexistent");
    let err = Describe {
        commit_hash: fake,
        max_commits_to_search: 10,
    }
    .execute(&mut store, &mut refs)
    .unwrap_err();
    assert!(matches!(err, VctrlError::NotFound(_)));
}
