//! Core behavior contracts (traits) for the `libvctrl_handler` version control
//! system.
//!
//! # Purpose
//! This module defines the abstract interfaces that concrete version control
//! backends must implement. By defining these as traits, the core data types
//! (in [`crate::types`]) remain completely decoupled from storage, networking,
//! and serialization logic.
//!
//! # Design rationale
//! The traits are split by responsibility:
//! - [`ObjectStore`] and [`RefStore`] handle persistence.
//! - [`Encoder`] and [`Decoder`] handle serialization.
//! - [`Hasher`] handles content addressing.
//! - [`Signer`] and [`Verifier`] handle cryptographic integrity.
//! - [`Transport`] handles remote synchronization.
//!
//! This separation of concerns allows mixing and matching implementations
//! (e.g., an in-memory store with a binary encoder) and vastly simplifies
//! unit testing of individual components.

use crate::errors::VctrlError;
use crate::types::{Blob, Commit, Hash, Tag, Tree};

/// Defines the interface for a content-addressable object database.
///
/// # Purpose
/// An `ObjectStore` is responsible for storing and retrieving raw, serialized
/// version control objects (blobs, trees, commits, tags) using their
/// [`Hash`] as the primary key.
///
/// # Design rationale
/// The trait uses `&Hash` for lookups rather than owned `Hash` values to
/// avoid unnecessary stack copies (64 bytes per hash). The `put` and `get`
/// methods deal with byte slices (`&[u8]` and `Vec<u8>`) rather than typed
/// objects, keeping the store agnostic to the serialization format.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Hash, ObjectStore, VctrlError};
/// use std::collections::HashMap;
///
/// #[derive(Default)]
/// struct InMemoryStore(HashMap<Hash, Vec<u8>>);
///
/// impl ObjectStore for InMemoryStore {
///     fn put(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError> {
///         self.0.insert(*hash, data.to_vec());
///         Ok(())
///     }
///     fn get(&self, hash: &Hash) -> Result<Vec<u8>, VctrlError> {
///         self.0.get(hash).cloned().ok_or_else(|| VctrlError::ObjectNotFound(*hash))
///     }
///     fn delete(&mut self, hash: &Hash) -> Result<(), VctrlError> {
///         self.0.remove(hash);
///         Ok(())
///     }
///     fn exists(&self, hash: &Hash) -> Result<bool, VctrlError> {
///         Ok(self.0.contains_key(hash))
///     }
/// }
///
/// let mut store = InMemoryStore::default();
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// store.put(&hash, b"data").unwrap();
/// assert!(store.exists(&hash).unwrap());
/// ```
pub trait ObjectStore {
    /// Stores a raw object in the database under the given hash.
    ///
    /// # Errors
    /// Returns [`VctrlError::IoError`] if the underlying storage fails to write.
    /// Returns [`VctrlError::Other`] for implementation-specific failures.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Hash, ObjectStore, VctrlError};
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct Store(HashMap<Hash, Vec<u8>>);
    /// # impl ObjectStore for Store {
    /// #     fn put(&mut self, h: &Hash, d: &[u8]) -> Result<(), VctrlError> { self.0.insert(*h, d.to_vec()); Ok(()) }
    /// #     fn get(&self, h: &Hash) -> Result<Vec<u8>, VctrlError> { self.0.get(h).cloned().ok_or_else(|| VctrlError::ObjectNotFound(*h)) }
    /// #     fn delete(&mut self, h: &Hash) -> Result<(), VctrlError> { self.0.remove(h); Ok(()) }
    /// #     fn exists(&self, h: &Hash) -> Result<bool, VctrlError> { Ok(self.0.contains_key(h)) }
    /// # }
    /// let mut s = Store::default();
    /// let h = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// s.put(&h, b"blob").unwrap();
    /// ```
    fn put(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError>;

    /// Retrieves a raw object from the database by its hash.
    ///
    /// # Errors
    /// Returns [`VctrlError::ObjectNotFound`] if no object exists for the hash.
    /// Returns [`VctrlError::IoError`] if the underlying storage fails to read.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Hash, ObjectStore, VctrlError};
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct Store(HashMap<Hash, Vec<u8>>);
    /// # impl ObjectStore for Store {
    /// #     fn put(&mut self, h: &Hash, d: &[u8]) -> Result<(), VctrlError> { self.0.insert(*h, d.to_vec()); Ok(()) }
    /// #     fn get(&self, h: &Hash) -> Result<Vec<u8>, VctrlError> { self.0.get(h).cloned().ok_or_else(|| VctrlError::ObjectNotFound(*h)) }
    /// #     fn delete(&mut self, h: &Hash) -> Result<(), VctrlError> { self.0.remove(h); Ok(()) }
    /// #     fn exists(&self, h: &Hash) -> Result<bool, VctrlError> { Ok(self.0.contains_key(h)) }
    /// # }
    /// let mut s = Store::default();
    /// let h = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// s.put(&h, b"blob").unwrap();
    /// assert_eq!(s.get(&h).unwrap(), b"blob");
    /// ```
    fn get(&self, hash: &Hash) -> Result<Vec<u8>, VctrlError>;

    /// Deletes an object from the database by its hash.
    ///
    /// # Errors
    /// Returns [`VctrlError::IoError`] if the underlying storage fails to delete.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Hash, ObjectStore, VctrlError};
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct Store(HashMap<Hash, Vec<u8>>);
    /// # impl ObjectStore for Store {
    /// #     fn put(&mut self, h: &Hash, d: &[u8]) -> Result<(), VctrlError> { self.0.insert(*h, d.to_vec()); Ok(()) }
    /// #     fn get(&self, h: &Hash) -> Result<Vec<u8>, VctrlError> { self.0.get(h).cloned().ok_or_else(|| VctrlError::ObjectNotFound(*h)) }
    /// #     fn delete(&mut self, h: &Hash) -> Result<(), VctrlError> { self.0.remove(h); Ok(()) }
    /// #     fn exists(&self, h: &Hash) -> Result<bool, VctrlError> { Ok(self.0.contains_key(h)) }
    /// # }
    /// let mut s = Store::default();
    /// let h = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// s.put(&h, b"blob").unwrap();
    /// s.delete(&h).unwrap();
    /// assert!(!s.exists(&h).unwrap());
    /// ```
    fn delete(&mut self, hash: &Hash) -> Result<(), VctrlError>;

    /// Checks if an object exists in the database.
    ///
    /// # Errors
    /// Returns [`VctrlError::IoError`] if the underlying storage fails to check.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Hash, ObjectStore, VctrlError};
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct Store(HashMap<Hash, Vec<u8>>);
    /// # impl ObjectStore for Store {
    /// #     fn put(&mut self, h: &Hash, d: &[u8]) -> Result<(), VctrlError> { self.0.insert(*h, d.to_vec()); Ok(()) }
    /// #     fn get(&self, h: &Hash) -> Result<Vec<u8>, VctrlError> { self.0.get(h).cloned().ok_or_else(|| VctrlError::ObjectNotFound(*h)) }
    /// #     fn delete(&mut self, h: &Hash) -> Result<(), VctrlError> { self.0.remove(h); Ok(()) }
    /// #     fn exists(&self, h: &Hash) -> Result<bool, VctrlError> { Ok(self.0.contains_key(h)) }
    /// # }
    /// let s = Store::default();
    /// let h = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// assert!(!s.exists(&h).unwrap());
    /// ```
    fn exists(&self, hash: &Hash) -> Result<bool, VctrlError>;
}

/// Defines the interface for a named reference store.
///
/// # Purpose
/// A `RefStore` maps human-readable names (e.g., "HEAD", "refs/heads/main")
/// to specific [`Hash`]es. This allows tracking branches and tags without
/// scanning the entire object database.
///
/// # Design rationale
/// References are stored separately from the [`ObjectStore`] because they
/// are mutable and frequently updated, whereas objects are immutable and
/// content-addressed.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Hash, RefStore, VctrlError};
/// use std::collections::HashMap;
///
/// #[derive(Default)]
/// struct InMemoryRefs(HashMap<String, Hash>);
///
/// impl RefStore for InMemoryRefs {
///     fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError> {
///         self.0.insert(name.to_string(), *hash);
///         Ok(())
///     }
///     fn get_ref(&self, name: &str) -> Result<Hash, VctrlError> {
///         self.0.get(name).copied().ok_or_else(|| VctrlError::RefNotFound(name.to_string()))
///     }
///     fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError> {
///         self.0.remove(name);
///         Ok(())
///     }
///     fn list_refs(&self) -> Result<Vec<String>, VctrlError> {
///         Ok(self.0.keys().cloned().collect())
///     }
/// }
///
/// let mut refs = InMemoryRefs::default();
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// refs.set_ref("main", &hash).unwrap();
/// assert_eq!(refs.get_ref("main").unwrap(), hash);
/// ```
pub trait RefStore {
    /// Sets or updates a named reference to point to a specific hash.
    ///
    /// # Errors
    /// Returns [`VctrlError::IoError`] if the underlying storage fails to write.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Hash, RefStore, VctrlError};
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct Refs(HashMap<String, Hash>);
    /// # impl RefStore for Refs {
    /// #     fn set_ref(&mut self, n: &str, h: &Hash) -> Result<(), VctrlError> { self.0.insert(n.to_string(), *h); Ok(()) }
    /// #     fn get_ref(&self, n: &str) -> Result<Hash, VctrlError> { self.0.get(n).copied().ok_or_else(|| VctrlError::RefNotFound(n.to_string())) }
    /// #     fn delete_ref(&mut self, n: &str) -> Result<(), VctrlError> { self.0.remove(n); Ok(()) }
    /// #     fn list_refs(&self) -> Result<Vec<String>, VctrlError> { Ok(self.0.keys().cloned().collect()) }
    /// # }
    /// let mut r = Refs::default();
    /// let h = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// r.set_ref("HEAD", &h).unwrap();
    /// ```
    fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError>;

    /// Retrieves the hash a named reference points to.
    ///
    /// # Errors
    /// Returns [`VctrlError::RefNotFound`] if the reference does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Hash, RefStore, VctrlError};
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct Refs(HashMap<String, Hash>);
    /// # impl RefStore for Refs {
    /// #     fn set_ref(&mut self, n: &str, h: &Hash) -> Result<(), VctrlError> { self.0.insert(n.to_string(), *h); Ok(()) }
    /// #     fn get_ref(&self, n: &str) -> Result<Hash, VctrlError> { self.0.get(n).copied().ok_or_else(|| VctrlError::RefNotFound(n.to_string())) }
    /// #     fn delete_ref(&mut self, n: &str) -> Result<(), VctrlError> { self.0.remove(n); Ok(()) }
    /// #     fn list_refs(&self) -> Result<Vec<String>, VctrlError> { Ok(self.0.keys().cloned().collect()) }
    /// # }
    /// let mut r = Refs::default();
    /// let h = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// r.set_ref("HEAD", &h).unwrap();
    /// assert_eq!(r.get_ref("HEAD").unwrap(), h);
    /// ```
    fn get_ref(&self, name: &str) -> Result<Hash, VctrlError>;

    /// Deletes a named reference.
    ///
    /// # Errors
    /// Returns [`VctrlError::IoError`] if the underlying storage fails to delete.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Hash, RefStore, VctrlError};
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct Refs(HashMap<String, Hash>);
    /// # impl RefStore for Refs {
    /// #     fn set_ref(&mut self, n: &str, h: &Hash) -> Result<(), VctrlError> { self.0.insert(n.to_string(), *h); Ok(()) }
    /// #     fn get_ref(&self, n: &str) -> Result<Hash, VctrlError> { self.0.get(n).copied().ok_or_else(|| VctrlError::RefNotFound(n.to_string())) }
    /// #     fn delete_ref(&mut self, n: &str) -> Result<(), VctrlError> { self.0.remove(n); Ok(()) }
    /// #     fn list_refs(&self) -> Result<Vec<String>, VctrlError> { Ok(self.0.keys().cloned().collect()) }
    /// # }
    /// let mut r = Refs::default();
    /// let h = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// r.set_ref("HEAD", &h).unwrap();
    /// r.delete_ref("HEAD").unwrap();
    /// assert!(r.get_ref("HEAD").is_err());
    /// ```
    fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError>;

    /// Lists all reference names currently stored.
    ///
    /// # Errors
    /// Returns [`VctrlError::IoError`] if the underlying storage fails to read
    /// the list of references.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Hash, RefStore, VctrlError};
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct Refs(HashMap<String, Hash>);
    /// # impl RefStore for Refs {
    /// #     fn set_ref(&mut self, n: &str, h: &Hash) -> Result<(), VctrlError> { self.0.insert(n.to_string(), *h); Ok(()) }
    /// #     fn get_ref(&self, n: &str) -> Result<Hash, VctrlError> { self.0.get(n).copied().ok_or_else(|| VctrlError::RefNotFound(n.to_string())) }
    /// #     fn delete_ref(&mut self, n: &str) -> Result<(), VctrlError> { self.0.remove(n); Ok(()) }
    /// #     fn list_refs(&self) -> Result<Vec<String>, VctrlError> { Ok(self.0.keys().cloned().collect()) }
    /// # }
    /// let mut r = Refs::default();
    /// let h = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// r.set_ref("main", &h).unwrap();
    /// r.set_ref("dev", &h).unwrap();
    /// let mut names = r.list_refs().unwrap();
    /// names.sort();
    /// assert_eq!(names, vec!["dev".to_string(), "main".to_string()]);
    /// ```
    fn list_refs(&self) -> Result<Vec<String>, VctrlError>;
}

/// Defines the interface for hashing raw data into a [`Hash`].
///
/// # Purpose
/// A `Hasher` implements the specific content-addressing algorithm (e.g.,
/// SHA-256, BLAKE3) used to identify objects in the system.
///
/// # Design rationale
/// The `hash` method does not return a `Result` because hashing pure byte
/// slices is an infallible operation. It takes `&self` to allow stateful
/// hashers or those initialized with specific keys.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Blob, Hash, Hasher};
///
/// struct DummyHasher;
/// impl Hasher for DummyHasher {
///     fn hash(&self, _data: &[u8]) -> Hash {
///         Hash::from_bytes(&[0u8; 64]).unwrap()
///     }
/// }
///
/// let hasher = DummyHasher;
/// let blob = Blob::new(b"hello".to_vec());
/// let hash = hasher.hash(blob.data());
/// assert_eq!(hash.as_bytes(), &[0u8; 64]);
/// ```
pub trait Hasher {
    /// Computes a cryptographic [`Hash`] from the provided byte slice.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Hash, Hasher};
    /// # struct HasherImpl;
    /// # impl Hasher for HasherImpl {
    /// #     fn hash(&self, _d: &[u8]) -> Hash { Hash::from_bytes(&[0u8; 64]).unwrap() }
    /// # }
    /// let hasher = HasherImpl;
    /// let hash = hasher.hash(b"data");
    /// assert_eq!(hash.as_bytes().len(), 64);
    /// ```
    #[must_use]
    fn hash(&self, data: &[u8]) -> Hash;
}

/// Defines the interface for serializing version control objects.
///
/// # Purpose
/// An `Encoder` translates in-memory data structures like [`Blob`] and
/// [`Commit`] into byte vectors suitable for storage in an [`ObjectStore`]
/// or transmission via a [`Transport`].
///
/// # Design rationale
/// The trait provides separate methods for each object type rather than a
/// generic `encode<T>(&self, obj: &T)` to avoid requiring objects to implement
/// a shared trait, keeping the data structs pure and decoupled.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Blob, Commit, Encoder, Tag, Tree, VctrlError};
///
/// struct DummyEncoder;
/// impl Encoder for DummyEncoder {
///     fn encode_blob(&self, blob: &Blob) -> Result<Vec<u8>, VctrlError> {
///         Ok(blob.data().to_vec())
///     }
///     fn encode_tree(&self, _tree: &Tree) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
///     fn encode_commit(&self, _commit: &Commit) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
///     fn encode_tag(&self, _tag: &Tag) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
/// }
///
/// let encoder = DummyEncoder;
/// let blob = Blob::new(b"data".to_vec());
/// assert_eq!(encoder.encode_blob(&blob).unwrap(), b"data");
/// ```
pub trait Encoder {
    /// Encodes a [`Blob`] into its serialized byte representation.
    ///
    /// # Errors
    /// Returns [`VctrlError::SerializationError`] if the encoder fails to
    /// serialize the blob.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Blob, Commit, Encoder, Tag, Tree, VctrlError};
    /// # struct EncoderImpl;
    /// # impl Encoder for EncoderImpl {
    /// #     fn encode_blob(&self, b: &Blob) -> Result<Vec<u8>, VctrlError> { Ok(b.data().to_vec()) }
    /// #     fn encode_tree(&self, _t: &Tree) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// #     fn encode_commit(&self, _c: &Commit) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// #     fn encode_tag(&self, _t: &Tag) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// # }
    /// let encoder = EncoderImpl;
    /// let blob = Blob::new(b"data".to_vec());
    /// assert_eq!(encoder.encode_blob(&blob).unwrap(), b"data");
    /// ```
    fn encode_blob(&self, blob: &Blob) -> Result<Vec<u8>, VctrlError>;

    /// Encodes a [`Tree`] into its serialized byte representation.
    ///
    /// # Errors
    /// Returns [`VctrlError::SerializationError`] if the encoder fails to
    /// serialize the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Blob, Commit, Encoder, EntryKind, Hash, Tag, Tree, TreeEntry, VctrlError};
    /// # struct EncoderImpl;
    /// # impl Encoder for EncoderImpl {
    /// #     fn encode_blob(&self, _b: &Blob) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// #     fn encode_tree(&self, t: &Tree) -> Result<Vec<u8>, VctrlError> { Ok(format!("{:?}", t.entries()).into_bytes()) }
    /// #     fn encode_commit(&self, _c: &Commit) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// #     fn encode_tag(&self, _t: &Tag) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// # }
    /// let encoder = EncoderImpl;
    /// let tree = Tree::new(vec![]).unwrap();
    /// assert!(encoder.encode_tree(&tree).is_ok());
    /// ```
    fn encode_tree(&self, tree: &Tree) -> Result<Vec<u8>, VctrlError>;

    /// Encodes a [`Commit`] into its serialized byte representation.
    ///
    /// # Errors
    /// Returns [`VctrlError::SerializationError`] if the encoder fails to
    /// serialize the commit.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Blob, Commit, Encoder, Hash, Tag, Tree, UserID, VctrlError};
    /// # struct EncoderImpl;
    /// # impl Encoder for EncoderImpl {
    /// #     fn encode_blob(&self, _b: &Blob) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// #     fn encode_tree(&self, _t: &Tree) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// #     fn encode_commit(&self, _c: &Commit) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// #     fn encode_tag(&self, _t: &Tag) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// # }
    /// let encoder = EncoderImpl;
    /// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let user = UserID::new("a".to_string(), "b".to_string()).unwrap();
    /// let commit = Commit::new(tree, vec![], user.clone(), user, "msg".to_string());
    /// assert!(encoder.encode_commit(&commit).is_ok());
    /// ```
    fn encode_commit(&self, commit: &Commit) -> Result<Vec<u8>, VctrlError>;

    /// Encodes a [`Tag`] into its serialized byte representation.
    ///
    /// # Errors
    /// Returns [`VctrlError::SerializationError`] if the encoder fails to
    /// serialize the tag.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Blob, Commit, Encoder, Hash, Tag, Tree, VctrlError};
    /// # struct EncoderImpl;
    /// # impl Encoder for EncoderImpl {
    /// #     fn encode_blob(&self, _b: &Blob) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// #     fn encode_tree(&self, _t: &Tree) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// #     fn encode_commit(&self, _c: &Commit) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// #     fn encode_tag(&self, _t: &Tag) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// # }
    /// let encoder = EncoderImpl;
    /// let target = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let tag = Tag::new("v1".to_string(), target, None, "msg".to_string()).unwrap();
    /// assert!(encoder.encode_tag(&tag).is_ok());
    /// ```
    fn encode_tag(&self, tag: &Tag) -> Result<Vec<u8>, VctrlError>;
}

/// Defines the interface for deserializing version control objects.
///
/// # Purpose
/// A `Decoder` translates byte vectors back into in-memory data structures.
/// It is the inverse of [`Encoder`].
///
/// # Design rationale
/// Decoding can fail due to corrupted data, malformed inputs, or version
/// mismatches, hence every method returns a `Result` with [`VctrlError`].
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Blob, Commit, Decoder, Hash, Tag, Tree, UserID, VctrlError};
///
/// struct DummyDecoder;
/// impl Decoder for DummyDecoder {
///     fn decode_blob(&self, data: &[u8]) -> Result<Blob, VctrlError> {
///         Ok(Blob::new(data.to_vec()))
///     }
///     fn decode_tree(&self, _data: &[u8]) -> Result<Tree, VctrlError> { Tree::new(vec![]) }
///     fn decode_commit(&self, _data: &[u8]) -> Result<Commit, VctrlError> {
///         let tree = Hash::from_bytes(&[0u8; 64])?;
///         let user = UserID::new("a".to_string(), "b".to_string())?;
///         Ok(Commit::new(tree, vec![], user.clone(), user, String::new()))
///     }
///     fn decode_tag(&self, _data: &[u8]) -> Result<Tag, VctrlError> {
///         let target = Hash::from_bytes(&[0u8; 64])?;
///         Tag::new("tag".to_string(), target, None, String::new())
///     }
/// }
///
/// let decoder = DummyDecoder;
/// let blob = decoder.decode_blob(b"data").unwrap();
/// assert_eq!(blob.data(), b"data");
/// ```
pub trait Decoder {
    /// Decodes a byte slice into a [`Blob`].
    ///
    /// # Errors
    /// Returns [`VctrlError::CorruptedData`] if the bytes are malformed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Blob, Commit, Decoder, Hash, Tag, Tree, UserID, VctrlError};
    /// # struct DecoderImpl;
    /// # impl Decoder for DecoderImpl {
    /// #     fn decode_blob(&self, d: &[u8]) -> Result<Blob, VctrlError> { Ok(Blob::new(d.to_vec())) }
    /// #     fn decode_tree(&self, _d: &[u8]) -> Result<Tree, VctrlError> { Tree::new(vec![]) }
    /// #     fn decode_commit(&self, _d: &[u8]) -> Result<Commit, VctrlError> { let t = Hash::from_bytes(&[0u8; 64])?; let u = UserID::new("a".to_string(), "b".to_string())?; Ok(Commit::new(t, vec![], u.clone(), u, String::new())) }
    /// #     fn decode_tag(&self, _d: &[u8]) -> Result<Tag, VctrlError> { let t = Hash::from_bytes(&[0u8; 64])?; Tag::new("t".to_string(), t, None, String::new()) }
    /// # }
    /// let decoder = DecoderImpl;
    /// let blob = decoder.decode_blob(b"data").unwrap();
    /// assert_eq!(blob.size(), 4);
    /// ```
    fn decode_blob(&self, data: &[u8]) -> Result<Blob, VctrlError>;

    /// Decodes a byte slice into a [`Tree`].
    ///
    /// # Errors
    /// Returns [`VctrlError::CorruptedData`] if the bytes are malformed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Blob, Commit, Decoder, Hash, Tag, Tree, UserID, VctrlError};
    /// # struct DecoderImpl;
    /// # impl Decoder for DecoderImpl {
    /// #     fn decode_blob(&self, _d: &[u8]) -> Result<Blob, VctrlError> { Ok(Blob::new(vec![])) }
    /// #     fn decode_tree(&self, _d: &[u8]) -> Result<Tree, VctrlError> { Tree::new(vec![]) }
    /// #     fn decode_commit(&self, _d: &[u8]) -> Result<Commit, VctrlError> { let t = Hash::from_bytes(&[0u8; 64])?; let u = UserID::new("a".to_string(), "b".to_string())?; Ok(Commit::new(t, vec![], u.clone(), u, String::new())) }
    /// #     fn decode_tag(&self, _d: &[u8]) -> Result<Tag, VctrlError> { let t = Hash::from_bytes(&[0u8; 64])?; Tag::new("t".to_string(), t, None, String::new()) }
    /// # }
    /// let decoder = DecoderImpl;
    /// let tree = decoder.decode_tree(b"").unwrap();
    /// assert!(tree.entries().is_empty());
    /// ```
    fn decode_tree(&self, data: &[u8]) -> Result<Tree, VctrlError>;

    /// Decodes a byte slice into a [`Commit`].
    ///
    /// # Errors
    /// Returns [`VctrlError::CorruptedData`] if the bytes are malformed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Blob, Commit, Decoder, Hash, Tag, Tree, UserID, VctrlError};
    /// # struct DecoderImpl;
    /// # impl Decoder for DecoderImpl {
    /// #     fn decode_blob(&self, _d: &[u8]) -> Result<Blob, VctrlError> { Ok(Blob::new(vec![])) }
    /// #     fn decode_tree(&self, _d: &[u8]) -> Result<Tree, VctrlError> { Tree::new(vec![]) }
    /// #     fn decode_commit(&self, _d: &[u8]) -> Result<Commit, VctrlError> { let t = Hash::from_bytes(&[0u8; 64])?; let u = UserID::new("a".to_string(), "b".to_string())?; Ok(Commit::new(t, vec![], u.clone(), u, String::new())) }
    /// #     fn decode_tag(&self, _d: &[u8]) -> Result<Tag, VctrlError> { let t = Hash::from_bytes(&[0u8; 64])?; Tag::new("t".to_string(), t, None, String::new()) }
    /// # }
    /// let decoder = DecoderImpl;
    /// let commit = decoder.decode_commit(b"").unwrap();
    /// assert_eq!(commit.message(), "");
    /// ```
    fn decode_commit(&self, data: &[u8]) -> Result<Commit, VctrlError>;

    /// Decodes a byte slice into a [`Tag`].
    ///
    /// # Errors
    /// Returns [`VctrlError::CorruptedData`] if the bytes are malformed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Blob, Commit, Decoder, Hash, Tag, Tree, UserID, VctrlError};
    /// # struct DecoderImpl;
    /// # impl Decoder for DecoderImpl {
    /// #     fn decode_blob(&self, _d: &[u8]) -> Result<Blob, VctrlError> { Ok(Blob::new(vec![])) }
    /// #     fn decode_tree(&self, _d: &[u8]) -> Result<Tree, VctrlError> { Tree::new(vec![]) }
    /// #     fn decode_commit(&self, _d: &[u8]) -> Result<Commit, VctrlError> { let t = Hash::from_bytes(&[0u8; 64])?; let u = UserID::new("a".to_string(), "b".to_string())?; Ok(Commit::new(t, vec![], u.clone(), u, String::new())) }
    /// #     fn decode_tag(&self, _d: &[u8]) -> Result<Tag, VctrlError> { let t = Hash::from_bytes(&[0u8; 64])?; Tag::new("t".to_string(), t, None, String::new()) }
    /// # }
    /// let decoder = DecoderImpl;
    /// let tag = decoder.decode_tag(b"").unwrap();
    /// assert_eq!(tag.name(), "t");
    /// ```
    fn decode_tag(&self, data: &[u8]) -> Result<Tag, VctrlError>;
}

/// Defines the interface for signing data cryptographically.
///
/// # Purpose
/// A `Signer` produces a cryptographic signature over a byte slice, typically
/// to attest to the authenticity of a [`Commit`] or [`Tag`].
///
/// # Design rationale
/// The trait returns a `Vec<u8>` to remain agnostic to the underlying
/// signature algorithm (e.g., Ed25519, RSA).
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Signer, VctrlError};
///
/// struct DummySigner;
/// impl Signer for DummySigner {
///     fn sign(&mut self, data: &[u8]) -> Result<Vec<u8>, VctrlError> {
///         Ok(data.to_vec())
///     }
/// }
///
/// let mut signer = DummySigner;
/// let sig = signer.sign(b"msg").unwrap();
/// assert_eq!(sig, b"msg");
/// ```
pub trait Signer {
    /// Signs the provided data, returning the signature as a byte vector.
    ///
    /// # Errors
    /// Returns [`VctrlError::Other`] if the signing process fails (e.g.,
    /// missing private key).
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Signer, VctrlError};
    /// # struct SignerImpl;
    /// # impl Signer for SignerImpl {
    /// #     fn sign(&mut self, d: &[u8]) -> Result<Vec<u8>, VctrlError> { Ok(d.to_vec()) }
    /// # }
    /// let mut signer = SignerImpl;
    /// let sig = signer.sign(b"data").unwrap();
    /// assert!(!sig.is_empty());
    /// ```
    fn sign(&mut self, data: &[u8]) -> Result<Vec<u8>, VctrlError>;
}

/// Defines the interface for verifying cryptographic signatures.
///
/// # Purpose
/// A `Verifier` checks whether a given byte slice and signature pair are valid
/// according to a specific cryptographic key.
///
/// # Design rationale
/// Returns `Result<bool, VctrlError>` rather than just `bool` to allow for
/// verification failures that are not strictly boolean (e.g., malformed
/// signature inputs or internal cryptographic errors).
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Verifier, VctrlError};
///
/// struct DummyVerifier;
/// impl Verifier for DummyVerifier {
///     fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, VctrlError> {
///         Ok(data == signature)
///     }
/// }
///
/// let verifier = DummyVerifier;
/// assert!(verifier.verify(b"msg", b"msg").unwrap());
/// assert!(!verifier.verify(b"msg", b"bad").unwrap());
/// ```
pub trait Verifier {
    /// Verifies a signature against the provided data.
    ///
    /// # Errors
    /// Returns [`VctrlError::Other`] if the verification process encounters
    /// an internal error (e.g., malformed signature).
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Verifier, VctrlError};
    /// # struct VerifierImpl;
    /// # impl Verifier for VerifierImpl {
    /// #     fn verify(&self, d: &[u8], s: &[u8]) -> Result<bool, VctrlError> { Ok(d == s) }
    /// # }
    /// let verifier = VerifierImpl;
    /// assert!(verifier.verify(b"data", b"data").unwrap());
    /// assert!(!verifier.verify(b"data", b"tampered").unwrap());
    /// ```
    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, VctrlError>;
}

/// Defines the interface for synchronizing objects with a remote backend.
///
/// # Purpose
/// A `Transport` abstracts the network or inter-process communication layer
/// required to fetch and push version control objects between a local
/// [`ObjectStore`] and a remote endpoint.
///
/// # Design rationale
/// `fetch_object` takes a `&Hash` to avoid copying the 64-byte key, while
/// `push_object` takes the raw bytes to be stored remotely. The trait is
/// distinct from [`ObjectStore`] to allow the local store to be disk-based
/// while the transport is purely network-oriented.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Hash, Transport, VctrlError};
/// use std::collections::HashMap;
///
/// #[derive(Default)]
/// struct InMemoryTransport(HashMap<Hash, Vec<u8>>);
///
/// impl Transport for InMemoryTransport {
///     fn fetch_object(&self, hash: &Hash) -> Result<Vec<u8>, VctrlError> {
///         self.0.get(hash).cloned().ok_or_else(|| VctrlError::ObjectNotFound(*hash))
///     }
///     fn push_object(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError> {
///         self.0.insert(*hash, data.to_vec());
///         Ok(())
///     }
/// }
///
/// let mut transport = InMemoryTransport::default();
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// transport.push_object(&hash, b"data").unwrap();
/// assert_eq!(transport.fetch_object(&hash).unwrap(), b"data");
/// ```
pub trait Transport {
    /// Fetches an object from the remote backend.
    ///
    /// # Errors
    /// Returns [`VctrlError::ObjectNotFound`] if the remote does not have the object.
    /// Returns [`VctrlError::IoError`] on network failures.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Hash, Transport, VctrlError};
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct TransportImpl(HashMap<Hash, Vec<u8>>);
    /// # impl Transport for TransportImpl {
    /// #     fn fetch_object(&self, h: &Hash) -> Result<Vec<u8>, VctrlError> { self.0.get(h).cloned().ok_or_else(|| VctrlError::ObjectNotFound(*h)) }
    /// #     fn push_object(&mut self, h: &Hash, d: &[u8]) -> Result<(), VctrlError> { self.0.insert(*h, d.to_vec()); Ok(()) }
    /// # }
    /// let mut t = TransportImpl::default();
    /// let h = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// t.push_object(&h, b"remote").unwrap();
    /// assert_eq!(t.fetch_object(&h).unwrap(), b"remote");
    /// ```
    fn fetch_object(&self, hash: &Hash) -> Result<Vec<u8>, VctrlError>;

    /// Pushes an object to the remote backend.
    ///
    /// # Errors
    /// Returns [`VctrlError::IoError`] on network failures or if the remote
    /// rejects the object.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Hash, Transport, VctrlError};
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct TransportImpl(HashMap<Hash, Vec<u8>>);
    /// # impl Transport for TransportImpl {
    /// #     fn fetch_object(&self, h: &Hash) -> Result<Vec<u8>, VctrlError> { self.0.get(h).cloned().ok_or_else(|| VctrlError::ObjectNotFound(*h)) }
    /// #     fn push_object(&mut self, h: &Hash, d: &[u8]) -> Result<(), VctrlError> { self.0.insert(*h, d.to_vec()); Ok(()) }
    /// # }
    /// let mut t = TransportImpl::default();
    /// let h = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// t.push_object(&h, b"payload").unwrap();
    /// assert!(t.fetch_object(&h).is_ok());
    /// ```
    fn push_object(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError>;
}
