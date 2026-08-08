//! Core abstractions that define the contract for every component.
//!
//! **No concrete implementations are allowed in this crate.**
//! These traits form the boundary between the fundamental definitions
//! and the actual implementations found in other crates
//! (e.g., `libvctrl_core`).
//!
//! # Architecture
//!
//! The `libvctrl` ecosystem is built around a set of **abstract capabilities**
//! rather than concrete types.  Each trait represents a single responsibility:
//!
//! | Trait | Responsibility | Typical implementor |
//! |---|---|---|
//! | [`ObjectStore`] | Content‑addressed key/value storage | In‑memory store, file‑system backend, database |
//! | [`RefStore`] | Mutable named pointers to objects | Branch & tag storage |
//! | [`Hasher`] | Cryptographic hash function | SHA‑512, SHA‑256, BLAKE3 |
//! | [`Encoder`] | Serialise domain objects to bytes | Binary format, Git‑compatible format |
//! | [`Decoder`] | Deserialise bytes back to domain objects | Same as Encoder |
//! | [`Signer`] | Create digital signatures | Ed25519, RSA, ECDSA |
//! | [`Verifier`] | Verify digital signatures | Same as Signer |
//! | [`Transport`] | Push/fetch raw object data across network | HTTP, SSH, custom protocol |
//!
//! By depending only on these traits, higher‑level logic (plumbing, porcelain)
//! becomes **backend‑agnostic** – you can swap storage backends, hashing
//! algorithms, serialisation formats, and network transports without changing
//! a single line of business logic.
//!
//! # The "everything is a trait" philosophy
//!
//! This crate does **not** provide a `Repository` struct with hard‑coded
//! behaviour.  Instead, you compose the traits you need.  Want a read‑only
//! repository that fetches objects over HTTP and verifies Ed25519 signatures?
//! Provide implementations of `ObjectStore` (that fetches on miss), `RefStore`,
//! `Hasher`, `Decoder`, `Verifier`, and `Transport`.  The plumbing functions
//! will accept those implementations and do the right thing.
//!
//! # Safety, errors, and panics
//!
//! - **No panics** – All operations return [`Result<T, VctrlError>`].
//!   Implementations must never panic (except on unrecoverable internal bugs,
//!   which should be `unreachable!`).
//! - **Validation** – Where validation is expected (names, hashes), the
//!   implementation must return the appropriate error; callers are
//!   **not** required to pre‑validate inputs (but may do so for efficiency).
//! - **Thread‑safety** – Traits do not mandate `Sync` or `Send`.  Implementors
//!   should add these bounds if their backend supports concurrent access.
//!   Consumers of the traits are encouraged to require `Send + Sync` only
//!   where strictly necessary.

use crate::errors::VctrlError;
use crate::types::{Blob, Commit, Hash, Tag, Tree};

// ============================================================================
// ObjectStore
// ============================================================================

/// A content‑addressable object store.
///
/// This is the lowest‑level storage abstraction.  It maps a fixed‑length
/// [`Hash`] to a variable‑length byte sequence (the **object**).  There is
/// exactly one object per hash, and the hash is **cryptographically derived**
/// from the content (hence “content‑addressable”).
///
/// # Why content‑addressable?
///
/// Content‑addressable storage has two critical properties:
/// 1. **Integrity** – any modification to the data changes its hash.  There is
///    no way to silently corrupt an object without being detected.
/// 2. **Deduplication** – identical data produces identical hashes, so storing
///    the same content twice (e.g., identical files in different commits) costs
///    nothing.
///
/// # Preconditions (what the caller must guarantee)
///
/// - `hash` *must* be a valid [`Hash`] (guaranteed by its constructor).
/// - `data` provided to [`put`](Self::put) *should* be the exact bytes that
///   produced `hash`.  The store does **not** verify this relationship.
///   Failing to uphold it will make objects irretrievable or incorrectly
///   addressable.
/// - `hash` passed to [`get`](Self::get) or [`exists`](Self::exists) must
///   have been obtained from a previous [`put`](Self::put) or from a trusted
///   source (e.g., a [`Tree`] entry).
///
/// # Postconditions (what the store guarantees after a successful operation)
///
/// - After a successful [`put`](Self::put), calling [`exists`](Self::exists)
///   with the same hash returns `Ok(true)` (unless the object was deleted in
///   the meantime).
/// - After a successful [`put`](Self::put), calling [`get`](Self::get) with
///   the same hash returns the **identical** `data` that was stored.
/// - After a successful [`delete`](Self::delete), [`exists`](Self::exists)
///   returns `Ok(false)` and [`get`](Self::get) returns
///   [`VctrlError::ObjectNotFound`].
///
/// # Idempotency
///
/// Storing the same `(hash, data)` pair multiple times should succeed
/// without error.  This makes batch operations simpler – an import script
/// can call `put` for every object without first checking whether it already
/// exists.
///
/// # Example (minimal in‑memory implementation)
///
/// ```rust
/// # use std::collections::HashMap;
/// use libvctrl_handler::*;
///
/// struct MemStore(HashMap<Hash, Vec<u8>>);
///
/// impl ObjectStore for MemStore {
///     fn put(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError> {
///         self.0.insert(*hash, data.to_vec());
///         Ok(())
///     }
///     fn get(&self, hash: &Hash) -> Result<Vec<u8>, VctrlError> {
///         self.0.get(hash).cloned().ok_or(VctrlError::ObjectNotFound(*hash))
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
    ///
    /// # Errors
    /// Returns an error if the write operation fails (e.g., disk full,
    /// permission denied).  Should never panic.
    fn put(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError>;

    /// Retrieve raw data by hash.
    ///
    /// # Errors
    /// Returns [`VctrlError::ObjectNotFound`] if no object with that hash
    /// exists.  Other errors (e.g., I/O) may also be returned.
    fn get(&self, hash: &Hash) -> Result<Vec<u8>, VctrlError>;

    /// Delete the object identified by `hash`.
    ///
    /// Deleting a non‑existent object should succeed silently (no error).
    /// This mirrors filesystem semantics and simplifies batch deletions.
    ///
    /// # Errors
    /// Returns an error if the deletion itself fails (e.g., permission denied).
    fn delete(&mut self, hash: &Hash) -> Result<(), VctrlError>;

    /// Check whether an object exists under the given hash.
    ///
    /// # Errors
    /// Returns an error if the existence check itself fails (e.g., the storage
    /// backend is unreachable).  A successful `Ok(false)` indicates only that
    /// the object is absent at this moment; it may be created later.
    fn exists(&self, hash: &Hash) -> Result<bool, VctrlError>;
}

// ============================================================================
// RefStore
// ============================================================================

/// A mutable mapping from human‑readable names to [`Hash`] values.
///
/// References are the **only** mutable primitive in `libvctrl`.  Objects
/// themselves are immutable (content‑addressed), but references let you
/// update which commit a branch points to, or which object a lightweight
/// tag targets.
///
/// # Typical usage
///
/// - Branches: `"refs/heads/main"` → commit hash
/// - Lightweight tags: `"refs/tags/v1.0"` → commit hash
/// - Special refs: `"HEAD"` → current branch name or commit hash
///
/// # Preconditions
///
/// - `name` must be non‑empty and ≤ [`MAX_NAME_LENGTH`](crate::MAX_NAME_LENGTH).
/// - `hash` must be a valid [`Hash`].
///
/// # Postconditions
///
/// - After a successful [`set_ref`](Self::set_ref), [`get_ref`](Self::get_ref)
///   with the same name returns the same hash (unless overwritten).
/// - [`list_refs`](Self::list_refs) returns every name that was successfully
///   set and not yet deleted.
///
/// # Implementation notes
///
/// Implementations **must** validate the name (length, emptiness) and return
/// [`VctrlError::InvalidName`] on failure.  The list returned by
/// [`list_refs`](Self::list_refs) may be in any order; callers must not
/// rely on ordering.
///
/// # Example (minimal in‑memory implementation)
///
/// ```rust
/// # use std::collections::HashMap;
/// use libvctrl_handler::*;
///
/// struct MemRefs(HashMap<String, Hash>);
///
/// impl RefStore for MemRefs {
///     fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError> {
///         if name.is_empty() || name.len() > MAX_NAME_LENGTH {
///             return Err(VctrlError::InvalidName(name.into()));
///         }
///         self.0.insert(name.to_string(), *hash);
///         Ok(())
///     }
///     fn get_ref(&self, name: &str) -> Result<Hash, VctrlError> {
///         self.0.get(name).copied().ok_or_else(|| VctrlError::RefNotFound(name.into()))
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
    /// Returns [`VctrlError::InvalidName`] if `name` is not valid (empty or too
    /// long).  Other errors (e.g., I/O) may also be returned.
    fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError>;

    /// Look up a reference by name.
    ///
    /// # Errors
    /// Returns [`VctrlError::RefNotFound`] if no reference with that name
    /// exists.
    fn get_ref(&self, name: &str) -> Result<Hash, VctrlError>;

    /// Delete a reference.
    ///
    /// Deleting a non‑existent reference should succeed silently.
    ///
    /// # Errors
    /// Returns an error if the deletion itself fails.
    fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError>;

    /// List all reference names currently stored.
    ///
    /// The returned list may be in any order.  Implementations should return
    /// the current snapshot at the time of the call; concurrent modifications
    /// are not required to be reflected.
    ///
    /// # Errors
    /// Returns an error if listing fails (e.g., I/O error while reading a
    /// directory).
    fn list_refs(&self) -> Result<Vec<String>, VctrlError>;
}

// ============================================================================
// Hasher
// ============================================================================

/// A cryptographically secure hash function.
///
/// This trait is the sole entry point for producing [`Hash`] values.  All
/// object identity in `libvctrl` flows through a `Hasher` implementation.
///
/// # Requirements
///
/// - **Deterministic** – `hash(data)` must always return the same `Hash` for
///   the same `data` (no nonce, no randomness).
/// - **Collision‑resistant** – it must be computationally infeasible to find
///   two different inputs that produce the same `Hash`.
/// - **Fixed output length** – the returned `Hash` must be exactly
///   [`HASH_LENGTH`](crate::HASH_LENGTH) bytes.
///
/// # Why SHA‑512?
///
/// `libvctrl` ships with a battle‑tested SHA‑512 implementation
/// (`libvctrl_sha512`).  You are free to provide your own hasher; as long as
/// it meets the contract, the entire ecosystem will work.
///
/// # Example (dummy hasher for illustration)
///
/// A real implementation would use a cryptographic library, but for
/// demonstration purposes we show a trivial (insecure!) hasher that always
/// returns the same zero‑filled hash.
///
/// ```rust
/// use libvctrl_handler::{Hash, Hasher, HASH_LENGTH};
///
/// struct DummyHasher;
/// impl Hasher for DummyHasher {
///     fn hash(&self, _data: &[u8]) -> Hash {
///         // NEVER do this in production! Always use a strong hash like SHA-512.
///         Hash::from_bytes(&[0u8; HASH_LENGTH]).expect("64 bytes")
///     }
/// }
///
/// let hasher = DummyHasher;
/// let h = hasher.hash(b"hello");
/// assert_eq!(h.as_bytes().len(), HASH_LENGTH);
/// ```
pub trait Hasher {
    /// Compute the hash of `data`.
    #[must_use]
    fn hash(&self, data: &[u8]) -> Hash;
}

// ============================================================================
// Encoder
// ============================================================================

/// Serialises high‑level objects into a byte representation.
///
/// The encoder is responsible for defining a **wire format** for objects.
/// It takes domain types ([`Blob`], [`Tree`], [`Commit`], [`Tag`]) and
/// produces a `Vec<u8>` that can be stored in an [`ObjectStore`] or
/// transmitted over a [`Transport`].
///
/// # Round‑trip contract
///
/// For any object `obj` and a matching [`Decoder`] implementation,
/// `decoder.decode_*(encoder.encode_*(obj)?)` must return `Ok(obj)`.
/// This is the fundamental guarantee that data survives storage.
///
/// # Format freedom
///
/// The exact binary format is **entirely up to the implementor**.  The
/// reference implementation in `libvctrl_core` uses a simple deterministic
/// binary format, but you could implement Git‑compatible formats, JSON,
/// CBOR, or anything else.  The only requirement is that a corresponding
/// `Decoder` can reverse the process.
///
/// # Example (simple identity encoder for Blob)
///
/// This example implements a trivial encoder that passes blobs through as-is.
/// Other object types are not implemented and return errors.
///
/// ```rust
/// use libvctrl_handler::{Blob, Commit, Encoder, Tag, Tree, VctrlError};
///
/// struct SimpleEncoder;
/// impl Encoder for SimpleEncoder {
///     fn encode_blob(&self, blob: &Blob) -> Result<Vec<u8>, VctrlError> {
///         Ok(blob.data().to_vec())
///     }
///     fn encode_tree(&self, _tree: &Tree) -> Result<Vec<u8>, VctrlError> {
///         Err(VctrlError::Other("tree encoding not implemented".into()))
///     }
///     fn encode_commit(&self, _commit: &Commit) -> Result<Vec<u8>, VctrlError> {
///         Err(VctrlError::Other("commit encoding not implemented".into()))
///     }
///     fn encode_tag(&self, _tag: &Tag) -> Result<Vec<u8>, VctrlError> {
///         Err(VctrlError::Other("tag encoding not implemented".into()))
///     }
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
/// The mirror image of [`Encoder`].  Given a byte slice that was produced by
/// a compatible encoder, a decoder must reconstruct the original object.
///
/// # Errors
///
/// Decoders are the **first line of defence** against data corruption and
/// malicious payloads.  Implementations must:
/// - Reject truncated or oversized data ([`VctrlError::CorruptedData`]).
/// - Reject data that violates structural invariants (e.g., unsorted tree
///   entries, invalid UTF‑8) – also [`VctrlError::CorruptedData`].
/// - Respect the DoS‑prevention limits defined in [`constants`](crate::constants)
///   ([`MAX_BLOB_SIZE`], [`MAX_TREE_ENTRIES`], [`MAX_MESSAGE_LENGTH`]).
///
/// # Example (simple identity decoder for Blob)
///
/// This example shows a minimal decoder that only handles blobs.
/// Other object types are not implemented and return errors.
///
/// ```rust
/// use libvctrl_handler::{Blob, Commit, Decoder, Tag, Tree, VctrlError};
///
/// struct SimpleDecoder;
/// impl Decoder for SimpleDecoder {
///     fn decode_blob(&self, data: &[u8]) -> Result<Blob, VctrlError> {
///         Ok(Blob::new(data.to_vec()))
///     }
///     fn decode_tree(&self, _data: &[u8]) -> Result<Tree, VctrlError> {
///         Err(VctrlError::Other("tree decoding not implemented".into()))
///     }
///     fn decode_commit(&self, _data: &[u8]) -> Result<Commit, VctrlError> {
///         Err(VctrlError::Other("commit decoding not implemented".into()))
///     }
///     fn decode_tag(&self, _data: &[u8]) -> Result<Tag, VctrlError> {
///         Err(VctrlError::Other("tag decoding not implemented".into()))
///     }
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
/// Used to produce cryptographic signatures over commit messages, tags, or
/// any other data.  The key material and algorithm are implementation‑defined.
///
/// # Contract
/// - `sign(data)` must produce a signature that can be verified by a
///   corresponding [`Verifier`].
/// - The same data + key must always produce the same signature (deterministic
///   algorithms like Ed25519) or at least produce a signature that verifies
///   correctly (randomised algorithms like ECDSA).
///
/// # Example (stub)
///
/// ```rust
/// use libvctrl_handler::{Signer, VctrlError};
///
/// struct StubSigner;
/// impl Signer for StubSigner {
///     fn sign(&self, data: &[u8]) -> Result<Vec<u8>, VctrlError> {
///         // In a real implementation, this would produce a cryptographic signature.
///         // Here we just return the data itself as a dummy signature.
///         Ok(data.to_vec())
///     }
/// }
/// ```
pub trait Signer {
    /// Sign the given data and return the signature.
    ///
    /// # Errors
    /// Returns an error if signing fails (e.g., missing key, algorithm error).
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, VctrlError>;
}

// ============================================================================
// Verifier
// ============================================================================

/// A digital signature verifier.
///
/// Matches a [`Signer`] implementation.  Given data and a signature, returns
/// `Ok(true)` if the signature is valid, `Ok(false)` if it is not.
///
/// # Why not `Result<bool, …>` where `false` is an error?
/// Because an invalid signature is not a system failure – it's an expected
/// outcome during verification.  Errors should be reserved for cases where
/// the verifier cannot operate (e.g., missing key, corrupted key).
///
/// # Example (stub)
///
/// ```rust
/// use libvctrl_handler::{Verifier, VctrlError};
///
/// struct StubVerifier;
/// impl Verifier for StubVerifier {
///     fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, VctrlError> {
///         // Dummy check: signature is valid iff it equals the data.
///         Ok(data == signature)
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

/// Object transport between repositories.
///
/// This trait abstracts the communication channel used to send and receive
/// raw object data.  It does not handle authentication, encryption, or
/// negotiation – those are implementation details.
///
/// # Concurrent fetches
/// `fetch_object` takes `&self` (not `&mut self`) to allow multiple threads
/// to fetch objects simultaneously.  Implementations that need mutable state
/// (e.g., an HTTP connection pool) should use interior mutability
/// (`Mutex`, `Atomic*`).
///
/// # Example (stub using a `HashMap`)
///
/// ```rust
/// # use std::collections::HashMap;
/// use libvctrl_handler::{Hash, Transport, VctrlError};
///
/// struct FakeTransport(HashMap<Hash, Vec<u8>>);
///
/// impl Transport for FakeTransport {
///     fn fetch_object(&self, hash: &Hash) -> Result<Vec<u8>, VctrlError> {
///         self.0.get(hash).cloned().ok_or(VctrlError::ObjectNotFound(*hash))
///     }
///     fn push_object(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError> {
///         self.0.insert(*hash, data.to_vec());
///         Ok(())
///     }
/// }
/// ```
pub trait Transport {
    /// Fetch the raw bytes of an object from a remote.
    ///
    /// # Errors
    /// Returns an error if the fetch operation fails (e.g., network error,
    /// object not found on remote).
    fn fetch_object(&self, hash: &Hash) -> Result<Vec<u8>, VctrlError>;

    /// Push raw bytes of an object to a remote.
    ///
    /// # Errors
    /// Returns an error if the push operation fails.
    fn push_object(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError>;
}
