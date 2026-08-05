mod common;
use common::*;
use libvctrl::*;

#[test]
fn stash_push_pop_list() {
    let mut store = MemoryStore::new();
    let mut refs = MemoryRefStore::new();

    let tree_hash = put_tree(&mut store, &Tree::new(vec![]).unwrap());

    let push = StashPush {
        tree_hash,
        author: alice(),
        message: Some("WIP".into()),
        encoder: Box::new(encoder()),
        hasher: Box::new(hasher()),
    };
    let hash = push.execute(&mut store, &mut refs).unwrap();
    assert!(store.exists(&hash).unwrap());

    let list = StashList.execute(&mut store, &mut refs).unwrap();
    assert_eq!(list.len(), 1);
    assert!(list[0].1 == hash);

    let popped = StashPop.execute(&mut store, &mut refs).unwrap();
    assert_eq!(popped, Some(tree_hash));

    let list = StashList.execute(&mut store, &mut refs).unwrap();
    assert_eq!(list.len(), 0);

    let empty = StashPop.execute(&mut store, &mut refs).unwrap();
    assert_eq!(empty, None);
}
