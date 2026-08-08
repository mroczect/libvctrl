use crate::errors::VctrlError;
use crate::types::{Blob, Commit, Hash, Tag, Tree};

pub trait ObjectStore {
    fn put(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError>;

    fn get(&self, hash: &Hash) -> Result<Vec<u8>, VctrlError>;

    fn delete(&mut self, hash: &Hash) -> Result<(), VctrlError>;

    fn exists(&self, hash: &Hash) -> Result<bool, VctrlError>;
}

pub trait RefStore {
    fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError>;

    fn get_ref(&self, name: &str) -> Result<Hash, VctrlError>;

    fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError>;

    fn list_refs(&self) -> Result<Vec<String>, VctrlError>;
}

pub trait Hasher {
    #[must_use]
    fn hash(&self, data: &[u8]) -> Hash;
}

pub trait Encoder {
    fn encode_blob(&self, blob: &Blob) -> Result<Vec<u8>, VctrlError>;

    fn encode_tree(&self, tree: &Tree) -> Result<Vec<u8>, VctrlError>;

    fn encode_commit(&self, commit: &Commit) -> Result<Vec<u8>, VctrlError>;

    fn encode_tag(&self, tag: &Tag) -> Result<Vec<u8>, VctrlError>;
}

pub trait Decoder {
    fn decode_blob(&self, data: &[u8]) -> Result<Blob, VctrlError>;

    fn decode_tree(&self, data: &[u8]) -> Result<Tree, VctrlError>;

    fn decode_commit(&self, data: &[u8]) -> Result<Commit, VctrlError>;

    fn decode_tag(&self, data: &[u8]) -> Result<Tag, VctrlError>;
}

pub trait Signer {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, VctrlError>;
}

pub trait Verifier {
    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, VctrlError>;
}

pub trait Transport {
    fn fetch_object(&self, hash: &Hash) -> Result<Vec<u8>, VctrlError>;

    fn push_object(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError>;
}
