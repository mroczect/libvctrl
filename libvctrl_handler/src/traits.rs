//! Core abstractions that define the contract for every component.
//!
//! **No concrete implementations are allowed in this crate.**
//! These traits form the boundary between the fundamental definitions
//! and the actual implementations found in other crates
//! (e.g., `libvctrl_core`).

use crate::errors::VctrlError;
use crate::types::{Blob, Commit, Hash, Tag, Tree};

/// A content‑addressable object store.
///
/// Implementations may store objects in memory, on disk, in a database,
/// or any other backend. The only requirement is that objects are
/// indexed by their [`Hash`].
pub trait ObjectStore {
    /// Store raw data under the given hash.
    ///
    /// # Errors
    /// Returns an error if the write operation fails.
    fn put(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError>;

    /// Retrieve raw data by hash.
    ///
    /// # Errors
    /// Returns [`VctrlError::ObjectNotFound`] if no object exists with that hash.
    fn get(&self, hash: &Hash) -> Result<Vec<u8>, VctrlError>;

    /// Delete the object identified by `hash`.
    ///
    /// # Errors
    /// Returns an error if the deletion fails.
    fn delete(&mut self, hash: &Hash) -> Result<(), VctrlError>;

    /// Check whether an object exists under the given hash.
    #[must_use]
    fn exists(&self, hash: &Hash) -> bool;
}

/// A reference store – maps names to [`Hash`] values.
///
/// References are typically used for branches, tags (lightweight),
/// or any symbolic name that points to a commit or other object.
pub trait RefStore {
    /// Create or update a reference.
    ///
    /// # Errors
    /// Returns [`VctrlError::InvalidName`] if `name` is not valid.
    fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError>;

    /// Look up a reference by name.
    ///
    /// # Errors
    /// Returns [`VctrlError::RefNotFound`] if the reference does not exist.
    fn get_ref(&self, name: &str) -> Result<Hash, VctrlError>;

    /// Delete a reference.
    ///
    /// # Errors
    /// Returns an error if the deletion fails.
    fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError>;

    /// List all reference names currently stored.
    ///
    /// # Errors
    /// Returns an error if the operation fails.
    fn list_refs(&self) -> Result<Vec<String>, VctrlError>;
}

/// A cryptographically secure hash function.
///
/// Implementations must return a [`Hash`] whose length is exactly
/// [`HASH_LENGTH`](crate::HASH_LENGTH) bytes.
pub trait Hasher {
    /// Compute the hash of `data`.
    #[must_use]
    fn hash(&self, data: &[u8]) -> Hash;
}

/// Serializes high‑level objects into a byte representation suitable for storage.
pub trait Encoder {
    /// Encode a [`Blob`] to bytes.
    ///
    /// # Errors
    /// Returns an error if encoding fails (e.g., invalid data).
    fn encode_blob(&self, blob: &Blob) -> Result<Vec<u8>, VctrlError>;

    /// Encode a [`Tree`] to bytes.
    ///
    /// # Errors
    /// Returns an error if encoding fails.
    fn encode_tree(&self, tree: &Tree) -> Result<Vec<u8>, VctrlError>;

    /// Encode a [`Commit`] to bytes.
    ///
    /// # Errors
    /// Returns an error if encoding fails.
    fn encode_commit(&self, commit: &Commit) -> Result<Vec<u8>, VctrlError>;

    /// Encode a [`Tag`] to bytes.
    ///
    /// # Errors
    /// Returns an error if encoding fails.
    fn encode_tag(&self, tag: &Tag) -> Result<Vec<u8>, VctrlError>;
}

/// Reconstructs objects from their byte representation.
pub trait Decoder {
    /// Decode a [`Blob`] from bytes.
    ///
    /// # Errors
    /// Returns an error if the data is corrupted or malformed.
    fn decode_blob(&self, data: &[u8]) -> Result<Blob, VctrlError>;

    /// Decode a [`Tree`] from bytes.
    ///
    /// # Errors
    /// Returns an error if the data is corrupted or malformed.
    fn decode_tree(&self, data: &[u8]) -> Result<Tree, VctrlError>;

    /// Decode a [`Commit`] from bytes.
    ///
    /// # Errors
    /// Returns an error if the data is corrupted or malformed.
    fn decode_commit(&self, data: &[u8]) -> Result<Commit, VctrlError>;

    /// Decode a [`Tag`] from bytes.
    ///
    /// # Errors
    /// Returns an error if the data is corrupted or malformed.
    fn decode_tag(&self, data: &[u8]) -> Result<Tag, VctrlError>;
}

/// A digital signature provider.
pub trait Signer {
    /// Sign the given data and return the signature.
    ///
    /// # Errors
    /// Returns an error if signing fails.
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, VctrlError>;
}

/// A digital signature verifier.
pub trait Verifier {
    /// Verify that `signature` is a valid signature for `data`.
    ///
    /// Returns `true` if the signature is valid, `false` otherwise.
    ///
    /// # Errors
    /// Returns an error if the verification process itself fails (e.g., invalid key).
    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, VctrlError>;
}

/// Object transport between repositories (fetch/push).
pub trait Transport {
    /// Fetch the raw bytes of an object identified by `hash` from a remote.
    ///
    /// # Errors
    /// Returns an error if the fetch operation fails (network, missing object, etc.).
    fn fetch_object(&mut self, hash: &Hash) -> Result<Vec<u8>, VctrlError>;

    /// Push raw bytes of an object to a remote.
    ///
    /// # Errors
    /// Returns an error if the push fails.
    fn push_object(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError>;
}
