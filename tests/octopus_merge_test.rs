mod common;
use common::*;
use libvctrl::*;

struct SimpleResolver;
impl ConflictResolver for SimpleResolver {
    fn resolve(&self, _base: &[u8], _ours: &[u8], _theirs: &[u8]) -> Option<Vec<u8>> {
        None
    }
}

#[test]
fn octopus_merge_two_branches() {
    let mut store = MemoryStore::new();
    let mut refs = MemoryRefStore::new();

    let tree_hash = put_tree(&mut store, &Tree::new(vec![]).unwrap());
    let base = Commit::new(tree_hash, vec![], alice(), alice(), "base".into(), None);
    let base_h = commit_hash(&base);
    store.put(&base_h, &Object::Commit(Box::new(base))).unwrap();

    let branch1 = Commit::new(tree_hash, vec![base_h], alice(), alice(), "b1".into(), None);
    let b1_h = commit_hash(&branch1);
    store
        .put(&b1_h, &Object::Commit(Box::new(branch1)))
        .unwrap();

    let branch2 = Commit::new(tree_hash, vec![base_h], alice(), alice(), "b2".into(), None);
    let b2_h = commit_hash(&branch2);
    store
        .put(&b2_h, &Object::Commit(Box::new(branch2)))
        .unwrap();

    let branch3 = Commit::new(tree_hash, vec![base_h], alice(), alice(), "b3".into(), None);
    let b3_h = commit_hash(&branch3);
    store
        .put(&b3_h, &Object::Commit(Box::new(branch3)))
        .unwrap();

    refs.set_ref("refs/heads/b1", &b1_h).unwrap();
    refs.set_ref("refs/heads/b2", &b2_h).unwrap();
    refs.set_ref("refs/heads/b3", &b3_h).unwrap();
    refs.set_head("refs/heads/b1").unwrap();

    let merge = OctopusMerge {
        branch_names: vec!["refs/heads/b2".into(), "refs/heads/b3".into()],
        author: alice(),
        committer: alice(),
        merger: Box::new(ThreeWayMerger),
        resolver: Box::new(SimpleResolver),
        encoder: Box::new(encoder()),
        hasher: Box::new(hasher()),
    };
    let result = merge.execute(&mut store, &mut refs).unwrap();
    let commit = store.get_commit(&result).unwrap();
    assert_eq!(commit.parents.len(), 3);
    assert!(commit.parents.contains(&b1_h));
    assert!(commit.parents.contains(&b2_h));
    assert!(commit.parents.contains(&b3_h));
}

#[test]
fn octopus_merge_too_many_parents() {
    let mut store = MemoryStore::new();
    let mut refs = MemoryRefStore::new();
    let tree_hash = put_tree(&mut store, &Tree::new(vec![]).unwrap());
    let base = Commit::new(tree_hash, vec![], alice(), alice(), "base".into(), None);
    let base_h = commit_hash(&base);
    store.put(&base_h, &Object::Commit(Box::new(base))).unwrap();

    let mut branches = Vec::new();
    for i in 0..256 {
        let b = Commit::new(
            tree_hash,
            vec![base_h],
            alice(),
            alice(),
            format!("b{}", i),
            None,
        );
        let h = commit_hash(&b);
        store.put(&h, &Object::Commit(Box::new(b))).unwrap();
        let name = format!("refs/heads/b{}", i);
        refs.set_ref(&name, &h).unwrap();
        branches.push(name);
    }
    refs.set_head("refs/heads/b0").unwrap();

    let merge = OctopusMerge {
        branch_names: branches[1..].to_vec(),
        author: alice(),
        committer: alice(),
        merger: Box::new(ThreeWayMerger),
        resolver: Box::new(SimpleResolver),
        encoder: Box::new(encoder()),
        hasher: Box::new(hasher()),
    };
    let err = merge.execute(&mut store, &mut refs).unwrap_err();
    assert!(err.to_string().contains("too many parents"));
}
