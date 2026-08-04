use crate::domain::hash::Hash;
use crate::hashing::Hasher;
use sha2::{Digest, Sha512};

pub struct Sha512Hasher;

impl Hasher for Sha512Hasher {
    fn hash_blob(&self, data: &[u8]) -> Hash {
        let mut h = Sha512::new();
        h.update(b"blob ");
        h.update((data.len() as u64).to_be_bytes());
        h.update(b"\0");
        h.update(data);
        Hash::from_bytes(h.finalize().into())
    }

    fn hash_tree_encoded(&self, data: &[u8]) -> Hash {
        let mut h = Sha512::new();
        h.update(b"tree ");
        h.update((data.len() as u64).to_be_bytes());
        h.update(b"\0");
        h.update(data);
        Hash::from_bytes(h.finalize().into())
    }

    fn hash_commit_encoded(&self, data: &[u8]) -> Hash {
        let mut h = Sha512::new();
        h.update(b"commit ");
        h.update((data.len() as u64).to_be_bytes());
        h.update(b"\0");
        h.update(data);
        Hash::from_bytes(h.finalize().into())
    }
}
