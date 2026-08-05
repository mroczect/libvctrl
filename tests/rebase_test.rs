mod common;
use common::*;
use libvctrl::*;

#[test]
fn rebase_simple() {
    let mut store = MemoryStore::new();
    let mut refs = MemoryRefStore::new();

    let tree_hash = put_tree(&mut store, &Tree::new(vec![]).unwrap());
    let base = Commit::new(tree_hash, vec![], alice(), alice(), "base".into(), None);
    let base_h = commit_hash(&base);
    store.put(&base_h, &Object::Commit(Box::new(base))).unwrap();

    let onto = Commit::new(
        tree_hash,
        vec![base_h],
        alice(),
        alice(),
        "onto".into(),
        None,
    );
    let onto_h = commit_hash(&onto);
    store.put(&onto_h, &Object::Commit(Box::new(onto))).unwrap();

    let c1 = Commit::new(tree_hash, vec![base_h], alice(), alice(), "c1".into(), None);
    let c1_h = commit_hash(&c1);
    store.put(&c1_h, &Object::Commit(Box::new(c1))).unwrap();
    let c2 = Commit::new(tree_hash, vec![c1_h], alice(), alice(), "c2".into(), None);
    let c2_h = commit_hash(&c2);
    store.put(&c2_h, &Object::Commit(Box::new(c2))).unwrap();

    refs.set_ref("refs/heads/main", &c2_h).unwrap();
    refs.set_head("refs/heads/main").unwrap();

    let rebase = Rebase {
        upstream: base_h,
        onto: onto_h,
        author: bob(),
        committer: bob(),
        merger: Box::new(ThreeWayMerger),
        resolver: Box::new(SimpleResolver),
        encoder: Box::new(encoder()),
        hasher: Box::new(hasher()),
    };
    let new_head = rebase.execute(&mut store, &mut refs).unwrap();

    let walk = RevWalk::new(&store, &[new_head]).unwrap();
    let commits: Vec<_> = walk.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(commits.len(), 4);
}

struct SimpleResolver;
impl ConflictResolver for SimpleResolver {
    fn resolve(&self, _: &[u8], _: &[u8], _: &[u8]) -> Option<Vec<u8>> {
        None
    }
}
