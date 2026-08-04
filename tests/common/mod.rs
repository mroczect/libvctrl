#![allow(dead_code)]

use libvctrl::{
    BinaryEncoder, Encoder, Hash, Hasher, MemoryRefStore, MemoryStore, ObjectStore, Sha512Hasher,
    UserInfo,
};

pub fn setup_store() -> MemoryStore {
    MemoryStore::new()
}

pub fn setup_refs() -> MemoryRefStore {
    MemoryRefStore::new()
}

pub fn encoder() -> BinaryEncoder {
    BinaryEncoder
}

pub fn hasher() -> Sha512Hasher {
    Sha512Hasher
}

pub fn user(name: &str, email: &str) -> UserInfo {
    UserInfo::new(name.to_string(), email.to_string())
}

pub fn alice() -> UserInfo {
    user("Alice", "alice@example.com")
}

pub fn bob() -> UserInfo {
    user("Bob", "bob@example.com")
}

pub fn blob_hash(data: &[u8]) -> Hash {
    hasher().hash_blob(data)
}

pub fn tree_hash(tree: &libvctrl::Tree) -> Hash {
    let encoder = encoder();
    let hasher = hasher();
    let mut buf = Vec::new();
    encoder.encode_tree(tree, &mut buf);
    hasher.hash_tree_encoded(&buf)
}

pub fn commit_hash(commit: &libvctrl::Commit) -> Hash {
    let encoder = encoder();
    let hasher = hasher();
    let mut buf = Vec::new();
    encoder.encode_commit(commit, &mut buf);
    hasher.hash_commit_encoded(&buf)
}

pub fn put_blob(store: &mut MemoryStore, data: &[u8]) -> Hash {
    let blob = libvctrl::Blob::new(data.to_vec());
    let hash = blob_hash(data);
    store.put(&hash, &libvctrl::Object::Blob(blob)).unwrap();
    hash
}

pub fn put_tree(store: &mut MemoryStore, tree: &libvctrl::Tree) -> Hash {
    let hash = tree_hash(tree);
    store
        .put(&hash, &libvctrl::Object::Tree(tree.clone()))
        .unwrap();
    hash
}
