pub mod sha512;
pub use sha512::*;

use crate::domain::hash::Hash;

pub trait Hasher {
    fn hash_blob(&self, data: &[u8]) -> Hash;
    fn hash_tree_encoded(&self, data: &[u8]) -> Hash;
    fn hash_commit_encoded(&self, data: &[u8]) -> Hash;
    fn hash_tag_encoded(&self, data: &[u8]) -> Hash;
}
