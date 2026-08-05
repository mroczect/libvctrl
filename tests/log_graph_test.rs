mod common;
use common::*;
use libvctrl::*;

#[test]
fn log_graph_basic() {
    let mut store = MemoryStore::new();
    let mut refs = MemoryRefStore::new();

    let tree_hash = put_tree(&mut store, &Tree::new(vec![]).unwrap());
    let c1 = Commit::new(tree_hash, vec![], alice(), alice(), "first".into(), None);
    let h1 = commit_hash(&c1);
    let c2 = Commit::new(tree_hash, vec![h1], alice(), alice(), "second".into(), None);
    let h2 = commit_hash(&c2);
    store.put(&h1, &Object::Commit(Box::new(c1))).unwrap();
    store.put(&h2, &Object::Commit(Box::new(c2))).unwrap();

    let cmd = LogGraph { head: h2 };
    let graph = cmd.execute(&mut store, &mut refs).unwrap();
    assert_eq!(graph.len(), 2);
    assert_eq!(graph[0].parent_indices, vec![1]);
    assert_eq!(graph[1].parent_indices, Vec::<usize>::new());
}
