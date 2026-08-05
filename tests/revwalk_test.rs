mod common;
use common::*;
use libvctrl::*;

#[test]
fn revwalk_returns_hash_commit_pairs() {
    let mut store = MemoryStore::new();
    let tree_hash = put_tree(&mut store, &Tree::new(vec![]).unwrap());
    let c1 = Commit::new(tree_hash, vec![], alice(), alice(), "first".into(), None);
    let h1 = commit_hash(&c1);
    let c2 = Commit::new(tree_hash, vec![h1], alice(), alice(), "second".into(), None);
    let h2 = commit_hash(&c2);
    store.put(&h1, &Object::Commit(Box::new(c1))).unwrap();
    store.put(&h2, &Object::Commit(Box::new(c2))).unwrap();

    let walk = RevWalk::new(&store, &[h2]).unwrap();
    let items: Vec<_> = walk.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].0, h2);
    assert_eq!(items[0].1.message, "second");
    assert_eq!(items[1].0, h1);
}
