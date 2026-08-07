//! Core abstractions that define the contract for every component.
//!
//! **No concrete implementations are allowed in this crate.**
//! These traits form the boundary between the fundamental definitions
//! and the actual implementations found in other crates
//! (e.g., `libvctrl_core`).

use crate::errors::VctrlError;
use crate::types::{Blob, Commit, Hash, Tag, Tree};

// ============================================================================
// ObjectStore
// ============================================================================
/// A content‑addressable object store.
///
/// Implementations may store objects in memory, on disk, in a database,
/// or any other backend. The only requirement is that objects are
/// indexed by their [`struct@Hash`].
///
/// # Preconditions (for callers)
/// - `hash` must be a valid [`struct@Hash`] (guaranteed by its constructor).
/// - `data` provided to [`put`](Self::put) should be the exact bytes
///   that produced `hash`; the store does **not** verify this relationship.
/// - `hash` passed to [`get`](Self::get) or [`exists`](Self::exists) must
///   have been obtained from a previous [`put`](Self::put) or from a trusted source.
///
/// # Postconditions (guarantees after successful operations)
/// - After a successful [`put`](Self::put), calling [`exists`](Self::exists) with the same
///   hash will return `Ok(true)` (unless the object was deleted in the meantime).
/// - After a successful [`put`](Self::put), calling [`get`](Self::get) with the same hash
///   will return the identical `data` that was stored.
/// - A successful [`delete`](Self::delete) will cause subsequent [`exists`](Self::exists)
///   to return `Ok(false)` and [`get`](Self::get) to return [`VctrlError::ObjectNotFound`].
///
/// # Implementation notes
/// - The store must be thread‑safe if shared across threads (this is left
///   to the implementor; the trait does not enforce `Sync` or `Send`).
/// - Implementations should treat [`put`](Self::put) as idempotent: storing the same
///   `(hash, data)` pair multiple times should not fail.
/// - The [`exists`](Self::exists) method is fallible because real storage backends
///   may encounter I/O errors. Implementations must never panic.
///
/// # Example (minimal in‑memory implementation)
/// ```rust,ignore
/// # use std::collections::HashMap;
/// # use libvctrl_handler::*;
/// #
/// struct MemStore(HashMap<Hash, Vec<u8>>);
///
/// impl ObjectStore for MemStore {
///     fn put(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError> {
///         self.0.insert(*hash, data.to_vec());
///         Ok(())
///     }
///     fn get(&self, hash: &Hash) -> Result<Vec<u8>, VctrlError> {
///         self.0.get(hash).cloned()
///               .ok_or(VctrlError::ObjectNotFound(*hash))
///     }
///     fn delete(&mut self, hash: &Hash) -> Result<(), VctrlError> {
///         self.0.remove(hash);
///         Ok(())
///     }
///     fn exists(&self, hash: &Hash) -> Result<bool, VctrlError> {
///         Ok(self.0.contains_key(hash))
///     }
/// }
/// ```
pub trait ObjectStore {
    /// Store raw data under the given hash.
    ///
    /// The store **does not** verify that `hash` is the actual hash of `data`.
    /// The caller must ensure the integrity of this relationship.
    /// Failing to do so may cause objects to become irretrievable or incorrectly
    /// addressable.
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
    ///
    /// # Errors
    /// Returns an error if the existence check itself fails (e.g., I/O error).
    fn exists(&self, hash: &Hash) -> Result<bool, VctrlError>;
}

// ============================================================================
// RefStore
// ============================================================================
/// A reference store – maps names to [`struct@Hash`] values.
///
/// References are typically used for branches, tags (lightweight),
/// or any symbolic name that points to a commit or other object.
///
/// # Preconditions
/// - `name` must be non‑empty and ≤ [`MAX_NAME_LENGTH`](crate::MAX_NAME_LENGTH).
/// - `hash` must be a valid [`struct@Hash`].
///
/// # Postconditions
/// - After a successful [`set_ref`](Self::set_ref), calling [`get_ref`](Self::get_ref)
///   with the same name must return the same hash (unless overwritten or deleted).
/// - [`list_refs`](Self::list_refs) must return every name that was successfully
///   set and not yet deleted.
///
/// # Implementation notes
/// - Implementations must validate the name (length, empty) and return
///   [`VctrlError::InvalidName`] on failure.
/// - The list returned by [`list_refs`](Self::list_refs) may be in any order.
///
/// # Example (minimal in‑memory implementation)
/// ```rust,ignore
/// # use std::collections::HashMap;
/// # use libvctrl_handler::*;
/// #
/// struct MemRefs(HashMap<String, Hash>);
///
/// impl RefStore for MemRefs {
///     fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError> {
///         if name.is_empty() || name.len() > MAX_NAME_LENGTH {
///             return Err(VctrlError::InvalidName(name.into()));
///         }
///         self.0.insert(name.into(), *hash);
///         Ok(())
///     }
///     fn get_ref(&self, name: &str) -> Result<Hash, VctrlError> {
///         self.0.get(name).copied()
///               .ok_or_else(|| VctrlError::RefNotFound(name.into()))
///     }
///     fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError> {
///         self.0.remove(name);
///         Ok(())
///     }
///     fn list_refs(&self) -> Result<Vec<String>, VctrlError> {
///         Ok(self.0.keys().cloned().collect())
///     }
/// }
/// ```
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

// ============================================================================
// Hasher
// ============================================================================
/// A cryptographically secure hash function.
///
/// Implementations must return a [`struct@Hash`] whose length is exactly
/// [`HASH_LENGTH`](crate::HASH_LENGTH) bytes.
///
/// # Contract
/// - The same input data must always produce the same hash.
/// - Different inputs should produce different hashes (collision resistance).
/// - The output length must be exactly [`HASH_LENGTH`](crate::HASH_LENGTH).
///
/// # Example (using SHA-512 from the `sha2` crate)
/// ```rust,ignore
/// # use sha2::{Sha512, Digest};
/// # use libvctrl_handler::*;
/// #
/// struct Sha512Hasher;
///
/// impl Hasher for Sha512Hasher {
///     fn hash(&self, data: &[u8]) -> Hash {
///         let digest = Sha512::digest(data);
///         Hash::from_bytes(&digest).expect("SHA-512 is 64 bytes")
///     }
/// }
/// ```
pub trait Hasher {
    /// Compute the hash of `data`.
    #[must_use]
    fn hash(&self, data: &[u8]) -> Hash;
}

// ============================================================================
// Encoder
// ============================================================================
/// Serializes high‑level objects into a byte representation suitable for storage.
///
/// # Round‑trip property
/// For any object `obj` of a given type, a valid [`Decoder`] implementation
/// must be able to reconstruct the original object from the bytes produced
/// by this encoder:
/// ```text
/// decoder.decode_*(encoder.encode_*(obj)?) == Ok(obj)
/// ```
/// The exact binary format is unspecified and left to the implementor.
///
/// # Errors
/// Methods may return [`VctrlError::SerializationError`] if the object
/// cannot be encoded (e.g., contains invalid data according to the format).
///
/// # Example (trivial identity encoder for Blob – not suitable for production)
/// ```rust,ignore
/// # use libvctrl_handler::*;
/// #
/// struct IdentityEncoder;
///
/// impl Encoder for IdentityEncoder {
///     fn encode_blob(&self, blob: &Blob) -> Result<Vec<u8>, VctrlError> {
///         Ok(blob.data().to_vec())
///     }
///     // ... other methods
/// #     fn encode_tree(&self, _: &Tree) -> Result<Vec<u8>, VctrlError> { todo!() }
/// #     fn encode_commit(&self, _: &Commit) -> Result<Vec<u8>, VctrlError> { todo!() }
/// #     fn encode_tag(&self, _: &Tag) -> Result<Vec<u8>, VctrlError> { todo!() }
/// }
/// ```
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

// ============================================================================
// Decoder
// ============================================================================
/// Reconstructs objects from their byte representation.
///
/// # Round‑trip property
/// For any valid encoded byte sequence produced by a corresponding [`Encoder`],
/// the decoder must return the original object.
///
/// # Errors
/// Methods must return [`VctrlError::CorruptedData`] or
/// [`VctrlError::SerializationError`] if the data is malformed, truncated,
/// or otherwise invalid.
///
/// # Example (trivial identity decoder for Blob)
/// ```rust,ignore
/// # use libvctrl_handler::*;
/// #
/// struct IdentityDecoder;
///
/// impl Decoder for IdentityDecoder {
///     fn decode_blob(&self, data: &[u8]) -> Result<Blob, VctrlError> {
///         Ok(Blob::new(data.to_vec()))
///     }
///     // ... other methods
/// #     fn decode_tree(&self, _: &[u8]) -> Result<Tree, VctrlError> { todo!() }
/// #     fn decode_commit(&self, _: &[u8]) -> Result<Commit, VctrlError> { todo!() }
/// #     fn decode_tag(&self, _: &[u8]) -> Result<Tag, VctrlError> { todo!() }
/// }
/// ```
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

// ============================================================================
// Signer
// ============================================================================
/// A digital signature provider.
///
/// # Contract
/// - [`sign`](Self::sign) must produce a deterministic or verifiable signature
///   for a given input and key (the key management is implementation‑defined).
/// - The signature must be verifiable by a corresponding [`Verifier`].
///
/// # Implementation notes
/// - The trait does not dictate the algorithm (Ed25519, RSA, etc.) or key storage.
/// - Implementations may be stateless (if the key is provided externally) or
///   stateful (if the signer holds the key internally).
///
/// # Example (stub)
/// ```rust,ignore
/// # use libvctrl_handler::*;
/// #
/// struct StubSigner;
///
/// impl Signer for StubSigner {
///     fn sign(&self, data: &[u8]) -> Result<Vec<u8>, VctrlError> {
///         // In a real implementation, this would produce a cryptographic signature.
///         Ok(data.to_vec()) // dummy signature
///     }
/// }
/// ```
pub trait Signer {
    /// Sign the given data and return the signature.
    ///
    /// # Errors
    /// Returns an error if signing fails.
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, VctrlError>;
}

// ============================================================================
// Verifier
// ============================================================================
/// A digital signature verifier.
///
/// # Contract
/// - [`verify`](Self::verify) must return `Ok(true)` if and only if the
///   signature is valid for the given data and the configured key.
/// - If the signature is invalid or doesn't match, it must return `Ok(false)`.
///
/// # Errors
/// Returns an error if the verification process itself cannot be completed
/// (e.g., corrupted key material, unsupported algorithm), not when the
/// signature is simply invalid.
///
/// # Example (stub)
/// ```rust,ignore
/// # use libvctrl_handler::*;
/// #
/// struct StubVerifier;
///
/// impl Verifier for StubVerifier {
///     fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, VctrlError> {
///         Ok(data == signature) // dummy check
///     }
/// }
/// ```
pub trait Verifier {
    /// Verify that `signature` is a valid signature for `data`.
    ///
    /// Returns `true` if the signature is valid, `false` otherwise.
    ///
    /// # Errors
    /// Returns an error if the verification process itself fails (e.g., invalid key).
    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, VctrlError>;
}

// ============================================================================
// Transport
// ============================================================================
/// Object transport between repositories (fetch/push).
///
/// # Contract
/// - [`fetch_object`](Self::fetch_object) must return the exact bytes that were
///   pushed via [`push_object`](Self::push_object) on the remote side.
/// - Both methods operate on raw bytes; no encoding/decoding is performed.
///
/// # Implementation notes
/// - The transport protocol (HTTP, SSH, custom) and authentication are
///   implementation details. The trait only represents the data transfer.
/// - Implementors should retry transient failures if appropriate, but
///   must surface permanent errors via the returned `Result`.
///
/// # Example (stub)
/// ```rust,ignore
/// # use std::collections::HashMap;
/// # use libvctrl_handler::*;
/// #
/// struct FakeTransport(HashMap<Hash, Vec<u8>>);
///
/// impl Transport for FakeTransport {
///     fn fetch_object(&mut self, hash: &Hash) -> Result<Vec<u8>, VctrlError> {
///         self.0.get(hash).cloned()
///               .ok_or(VctrlError::ObjectNotFound(*hash))
///     }
///     fn push_object(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError> {
///         self.0.insert(*hash, data.to_vec());
///         Ok(())
///     }
/// }
/// ```
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
