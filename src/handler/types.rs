use crate::handler::error::VctrlError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hash([u8; 64]);

impl Hash {
    pub const fn from_bytes(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, HashError> {
        let array: [u8; 64] = slice
            .try_into()
            .map_err(|_| HashError::InvalidLength(slice.len()))?;
        Ok(Self(array))
    }

    pub fn from_hex(hex: &str) -> Result<Self, HashError> {
        let bytes = hex::decode(hex).map_err(|_| HashError::InvalidHex)?;
        Self::from_slice(&bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash({})", self.to_hex())
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl FromStr for Hash {
    type Err = HashError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_hex(s)
    }
}

impl Serialize for Hash {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Hash::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum HashError {
    #[error("invalid hash length: {0} (expected 64)")]
    InvalidLength(usize),
    #[error("invalid hex string")]
    InvalidHex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blob {
    data: Vec<u8>,
}

impl Blob {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    pub fn hash(&self) -> Result<Hash, VctrlError> {
        hash_blob(&self.data)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryKind {
    Blob,
    Tree,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeEntry {
    pub name: String,
    pub kind: EntryKind,
    pub hash: Hash,
}

impl TreeEntry {
    pub fn new(name: String, kind: EntryKind, hash: Hash) -> Self {
        Self { name, kind, hash }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tree {
    entries: Vec<TreeEntry>,
}

impl Tree {
    pub fn new(mut entries: Vec<TreeEntry>) -> Result<Self, TreeError> {
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        for pair in entries.windows(2) {
            if pair[0].name == pair[1].name {
                return Err(TreeError::DuplicateEntry(pair[0].name.clone()));
            }
        }
        Ok(Self { entries })
    }

    pub fn entries(&self) -> &[TreeEntry] {
        &self.entries
    }

    pub fn into_entries(self) -> Vec<TreeEntry> {
        self.entries
    }

    pub fn hash(&self) -> Result<Hash, VctrlError> {
        let serialized = serde_json::to_vec(&self.entries).map_err(VctrlError::from)?;
        hash_tree(&serialized)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum TreeError {
    #[error("duplicate entry name: {0}")]
    DuplicateEntry(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub name: String,
    pub email: String,
}

impl UserInfo {
    pub fn new(name: String, email: String) -> Self {
        Self { name, email }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    tree: Hash,
    parents: Vec<Hash>,
    author: UserInfo,
    committer: UserInfo,
    timestamp: DateTime<Utc>,
    message: String,
    signature: Option<Vec<u8>>,
}

impl Commit {
    pub fn new(
        tree: Hash,
        parents: Vec<Hash>,
        author: UserInfo,
        committer: UserInfo,
        message: String,
        signature: Option<Vec<u8>>,
    ) -> Result<Self, VctrlError> {
        let commit = Self {
            tree,
            parents,
            author,
            committer,
            timestamp: Utc::now(),
            message,
            signature,
        };
        let _ = commit.hash()?;
        Ok(commit)
    }

    pub fn tree(&self) -> &Hash {
        &self.tree
    }
    pub fn parents(&self) -> &[Hash] {
        &self.parents
    }
    pub fn author(&self) -> &UserInfo {
        &self.author
    }
    pub fn committer(&self) -> &UserInfo {
        &self.committer
    }
    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }
    pub fn message(&self) -> &str {
        &self.message
    }
    pub fn signature(&self) -> Option<&[u8]> {
        self.signature.as_deref()
    }

    pub fn hash(&self) -> Result<Hash, VctrlError> {
        let serialized = serde_json::to_vec(self).map_err(VctrlError::from)?;
        hash_commit(&serialized)
    }
}

#[derive(Debug, Clone)]
pub enum Object {
    Blob(Blob),
    Tree(Tree),
    Commit(Box<Commit>),
}

impl Object {
    pub fn obj_type(&self) -> &str {
        match self {
            Object::Blob(_) => "blob",
            Object::Tree(_) => "tree",
            Object::Commit(_) => "commit",
        }
    }

    pub fn hash(&self) -> Result<Hash, VctrlError> {
        match self {
            Object::Blob(b) => b.hash(),
            Object::Tree(t) => t.hash(),
            Object::Commit(c) => c.hash(),
        }
    }
}

pub trait ObjectStore {
    fn put(&mut self, obj: &Object) -> Result<Hash, VctrlError>;
    fn get(&self, hash: &Hash) -> Result<Option<Object>, VctrlError>;
    fn exists(&self, hash: &Hash) -> Result<bool, VctrlError>;
}

pub trait RefStore {
    fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError>;
    fn get_ref(&self, name: &str) -> Result<Option<Hash>, VctrlError>;
    fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError>;
    fn set_head(&mut self, target: &str) -> Result<(), VctrlError>;
    fn head(&self) -> Result<Option<Hash>, VctrlError>;
}

pub(crate) fn hash_blob(data: &[u8]) -> Result<Hash, VctrlError> {
    let mut hasher = Sha512::new();
    hasher.update(b"blob ");
    hasher.update(data.len().to_string().as_bytes());
    hasher.update(b"\0");
    hasher.update(data);
    Ok(Hash::from_bytes(hasher.finalize().into()))
}

pub(crate) fn hash_tree(data: &[u8]) -> Result<Hash, VctrlError> {
    let mut hasher = Sha512::new();
    hasher.update(b"tree ");
    hasher.update(data.len().to_string().as_bytes());
    hasher.update(b"\0");
    hasher.update(data);
    Ok(Hash::from_bytes(hasher.finalize().into()))
}

pub(crate) fn hash_commit(data: &[u8]) -> Result<Hash, VctrlError> {
    let mut hasher = Sha512::new();
    hasher.update(b"commit ");
    hasher.update(data.len().to_string().as_bytes());
    hasher.update(b"\0");
    hasher.update(data);
    Ok(Hash::from_bytes(hasher.finalize().into()))
}

mod hex {
    pub fn decode(hex: &str) -> Result<Vec<u8>, ()> {
        if !hex.len().is_multiple_of(2) {
            return Err(());
        }
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| ()))
            .collect()
    }
}
