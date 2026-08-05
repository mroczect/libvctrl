use crate::domain::hash::Hash;
use crate::hashing::Hasher;

pub trait HashVerifier: Hasher {
    fn verify_blob(&self, hash: &Hash, data: &[u8]) -> bool {
        self.hash_blob(data) == *hash
    }
    fn verify_tree_encoded(&self, hash: &Hash, encoded: &[u8]) -> bool {
        self.hash_tree_encoded(encoded) == *hash
    }
    fn verify_commit_encoded(&self, hash: &Hash, encoded: &[u8]) -> bool {
        self.hash_commit_encoded(encoded) == *hash
    }
    fn verify_tag_encoded(&self, hash: &Hash, encoded: &[u8]) -> bool {
        self.hash_tag_encoded(encoded) == *hash
    }
}
impl<T: Hasher + ?Sized> HashVerifier for T {}
