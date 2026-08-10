//! # `libvctrl` - The Ultimate Version Control SDK
//!
//! **The all-in-one Version Control System (VCS) Software Development Kit.**
//!
//! This crate provides a unified, batteries-included interface for building
//! custom version control systems. It aggregates three foundational layers
//! into a single coherent namespace, allowing developers to bootstrap a fully
//! functional VCS backend without stitching multiple crates together manually.
//!
//! ## Architecture
//!
//! The SDK is composed of three re-exported sub-crates:
//!
//! 1. **Contracts** ([`handler`]): Core data types ([`Blob`], [`Commit`], [`Tree`]),
//!    behavior traits ([`ObjectStore`], [`Encoder`]), and error definitions
//!    ([`VctrlError`]). These are pure, dependency-light definitions.
//! 2. **Implementations** ([`mod@reference`]): Ready-to-use backends including an
//!    in-memory store ([`MemoryStore`]), binary encoder/decoder
//!    ([`BinaryEncoder`], [`BinaryDecoder`]), SHA-512 hasher adapter
//!    ([`Sha512Hasher`]), and ergonomic object builders ([`TreeBuilder`]).
//! 3. **Cryptography** ([`crypto`]): A pure-Rust, `no_std`-compatible SHA-512,
//!    HMAC-SHA-512, and HKDF-SHA-512 implementation.
//!
//! ## Design Rationale
//!
//! - **Facade Pattern**: By re-exporting the essential types at the root level,
//!   users can simply `use libvctrl::*;` without worrying about deep module
//!   paths. Complex internal dependencies are abstracted away.
//! - **Namespace Isolation**: To prevent name clashes (e.g., between the VCS
//!   [`struct@Hash`] type and the cryptographic [`crypto::Hash`]), the low-level
//!   cryptographic primitives are grouped under the [`crypto`] module.
//! - **Robustness**: All underlying crates enforce `#![forbid(unsafe_code)]`
//!   and strict Clippy lints, guaranteeing memory safety and high code quality
//!   across the entire stack.
//!
//! # Examples
//!
//! Building a tree, encoding it, hashing it, and storing it:
//!
//! ```
//! use libvctrl::{
//!     EntryKind, Hash, TreeBuilder, TreeEntryBuilder, BinaryEncoder, Sha512Hasher,
//!     MemoryStore, Encoder, Hasher, ObjectStore, VctrlError,
//! };
//! use std::io::Read;
//!
//! // 1. Build a Tree containing a single file entry
//! let blob_hash = Hash::from_bytes(&[0xAB; 64])?;
//! let entry = TreeEntryBuilder::new("file.txt".to_string(), EntryKind::Blob, blob_hash).build()?;
//! let tree = TreeBuilder::new().entry(entry).build()?;
//!
//! // 2. Encode the Tree into binary format
//! let encoder = BinaryEncoder;
//! let encoded_bytes = encoder.encode_tree(&tree)?;
//!
//! // 3. Hash the encoded bytes to get an address
//! let hasher = Sha512Hasher;
//! let tree_hash = hasher.hash(&encoded_bytes);
//!
//! // 4. Store the encoded object in memory
//! let mut store = MemoryStore::new();
//! store.put(&tree_hash, &encoded_bytes)?;
//!
//! // 5. Retrieve and verify the object
//! assert!(store.exists(&tree_hash)?);
//! let mut reader = store.get(&tree_hash)?;
//! let mut buf = Vec::new();
//! reader.read_to_end(&mut buf).map_err(VctrlError::IoError)?;
//! assert_eq!(buf, encoded_bytes);
//!
//! # Ok::<(), VctrlError>(())
//! ```

#![forbid(unsafe_code)]
#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    unused_qualifications
)]

// ---------------------------------------------------------------------------
// Sub-crate Re-exports
// ---------------------------------------------------------------------------

/// The core contracts (types, traits, errors) for the version control system.
///
/// # Purpose
/// This module exposes the pure data structures and behavior definitions
/// from `libvctrl_handler`. It acts as the foundational layer of the SDK,
/// defining *what* a version control object is and *how* it should behave,
/// without providing any actual storage or serialization logic.
///
/// # Design Rationale
/// Exposing this as a sub-module allows users to explicitly depend on the
/// contracts if they only need the type definitions (e.g., for a frontend
/// that doesn't implement storage), while still being part of the unified SDK.
///
/// # Examples
///
/// Accessing the `Blob` type via the `handler` module:
///
/// ```
/// use libvctrl::handler::Blob;
///
/// let blob = Blob::new(b"hello".to_vec());
/// assert_eq!(blob.size(), 5);
/// ```
pub use libvctrl_handler as handler;

/// The reference implementations (in-memory store, binary codec) for the VCS.
///
/// # Purpose
/// This module exposes the ready-to-use backends from `libvctrl_core`. It
/// provides concrete implementations for all the traits defined in the
/// [`handler`] module.
///
/// # Design Rationale
/// It serves as the "batteries-included" layer. Users can use these
/// implementations directly for rapid prototyping or testing, or use them as
/// reference examples when building custom backends (e.g., a disk-based store).
///
/// # Examples
///
/// Using the `MemoryStore` via the `reference` module:
///
/// ```
/// use libvctrl::MemoryStore;
/// use libvctrl::handler::{Hash, ObjectStore};
///
/// let mut store = MemoryStore::new();
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// store.put(&hash, b"data").unwrap();
/// assert!(store.exists(&hash).unwrap());
/// ```
pub use libvctrl_core as reference;

/// The cryptographic primitives (SHA-512, HMAC, HKDF).
///
/// # Purpose
/// This module exposes the pure-Rust cryptographic implementations from
/// `libvctrl_sha512`.
///
/// # Design Rationale
/// It is kept as a separate module to avoid naming conflicts with the VCS-level
/// [`struct@Hash`] type. The VCS `Hash` is a 64-byte array wrapper, whereas
/// [`crypto::Hash`] is the actual SHA-512 hasher state machine.
///
/// # Examples
///
/// Computing a SHA-512 hash:
///
/// ```
/// use libvctrl::crypto::Hash as Sha512Hash;
///
/// let digest = Sha512Hash::hash(b"hello world");
/// assert_eq!(digest.len(), 64);
/// ```
pub use libvctrl_sha512 as crypto;

// ---------------------------------------------------------------------------
// Root-level Re-exports (Contracts)
// ---------------------------------------------------------------------------

/// System-wide constants and structural limits.
///
/// # Purpose
/// Centralizes all magic numbers and structural limits used across the version
/// control system.
///
/// # Design Rationale
/// Defining them in a single module ensures that validation logic in type
/// constructors, encoders, and storage backends remains consistent and easily
/// tunable.
///
/// # Examples
///
/// ```
/// use libvctrl::constants::HASH_LENGTH;
/// assert_eq!(HASH_LENGTH, 64);
/// ```
pub use handler::constants;

/// Logical object type enumerations (e.g., [`EntryKind`]).
///
/// # Purpose
/// Distinguishes between files and directories at a high level, decoupled from
/// raw filesystem mode bits.
///
/// # Examples
///
/// ```
/// use libvctrl::enums::EntryKind;
/// assert_ne!(EntryKind::Blob, EntryKind::Tree);
/// ```
pub use handler::enums;

/// Unified error handling ([`VctrlError`]).
///
/// # Purpose
/// The single error type returned by all fallible operations in the SDK.
///
/// # Examples
///
/// ```
/// use libvctrl::errors::VctrlError;
/// let err = VctrlError::Other("fail".to_string());
/// assert_eq!(err.to_string(), "fail");
/// ```
pub use handler::errors;

/// Helper macros for ergonomic error construction.
///
/// # Purpose
/// Provides macros like [`vctrl_error_other!`] to simplify creating formatted
/// error messages. Note that the macro is exported at the crate root,
/// so you should import it as `use libvctrl::vctrl_error_other`.
///
/// # Examples
///
/// ```
/// use libvctrl::VctrlError;
/// use libvctrl_handler::vctrl_error_other;
///
/// let err: VctrlError = vctrl_error_other!("code {}", 500);
/// assert_eq!(err.to_string(), "code 500");
/// ```
pub use handler::macros;

/// Core behavior contracts (traits).
///
/// # Purpose
/// Defines the interfaces (e.g., [`ObjectStore`], [`Encoder`]) that any
/// concrete backend must implement.
///
/// # Examples
///
/// ```
/// use libvctrl::traits::Hasher;
/// use libvctrl::Hash;
///
/// struct DummyHasher;
/// impl Hasher for DummyHasher {
///     fn hash(&self, _data: &[u8]) -> Hash {
///         Hash::from_bytes(&[0u8; 64]).unwrap()
///     }
/// }
/// ```
pub use handler::traits;

/// Core data structures representing version control objects.
///
/// # Purpose
/// Contains the immutable domain models: [`Blob`], [`Tree`], [`Commit`], [`Tag`],
/// and supporting types like [`struct@Hash`] and [`UserID`].
///
/// # Examples
///
/// ```
/// use libvctrl::types::Blob;
/// let blob = Blob::new(vec![1, 2, 3]);
/// assert_eq!(blob.size(), 3);
/// ```
pub use handler::types;

/// Re-exports of fundamental system constants.
///
/// # Purpose
/// These constants (like [`HASH_LENGTH`] and [`MAX_BLOB_SIZE`]) are used so
/// frequently that they are re-exported at the crate root. This saves the caller
/// from having to write `libvctrl::constants::HASH_LENGTH` everywhere.
///
/// # Examples
///
/// ```
/// use libvctrl::HASH_LENGTH;
/// assert_eq!(HASH_LENGTH, 64);
/// ```
pub use handler::{
    HASH_LENGTH, MAX_BLOB_SIZE, MAX_MESSAGE_LENGTH, MAX_NAME_LENGTH, MAX_TREE_ENTRIES,
};

/// Re-export of the logical entry kind enum.
///
/// # Purpose
/// Used in [`TreeEntry`] to distinguish between a file ([`Blob`]) and a
/// subdirectory ([`Tree`]).
///
/// # Examples
///
/// ```
/// use libvctrl::EntryKind;
/// assert_eq!(EntryKind::Blob, EntryKind::Blob);
/// ```
pub use handler::EntryKind;

/// Re-export of the unified error type.
///
/// # Purpose
/// Every fallible operation in this SDK returns `Result<_, VctrlError>`.
/// Making it available at the root streamlines error handling.
///
/// # Examples
///
/// ```
/// use libvctrl::VctrlError;
/// let err = VctrlError::Other("test".to_string());
/// assert!(err.to_string().contains("test"));
/// ```
pub use handler::VctrlError;

/// Re-exports of the core behavior traits.
///
/// # Purpose
/// Provides direct access to the interfaces that define VCS behavior, such as
/// [`ObjectStore`] for persistence and [`Encoder`] for serialization.
///
/// # Examples
///
/// ```
/// use libvctrl::{Hasher, Hash};
///
/// struct MyHasher;
/// impl Hasher for MyHasher {
///     fn hash(&self, _data: &[u8]) -> Hash {
///         Hash::from_bytes(&[0u8; 64]).unwrap()
///     }
/// }
/// ```
pub use handler::{Decoder, Encoder, Hasher, ObjectStore, RefStore, Signer, Transport, Verifier};

/// Re-exports of the core data structures.
///
/// # Purpose
/// All version-control objects ([`Blob`], [`Tree`], [`Commit`], [`Tag`]) and
/// their supporting types are available directly from the crate root for
/// ergonomic access.
///
/// # Examples
///
/// ```
/// use libvctrl::Blob;
/// let blob = Blob::new(vec![1, 2, 3]);
/// assert_eq!(blob.size(), 3);
/// ```
pub use handler::{Blob, Commit, CommitMeta, Hash, Tag, Tree, TreeEntry, UserID};

// ---------------------------------------------------------------------------
// Root-level Re-exports (Reference Implementations)
// ---------------------------------------------------------------------------

/// Re-exports of binary serialization modules.
///
/// # Purpose
/// Provides the [`BinaryEncoder`] and [`BinaryDecoder`] which translate
/// in-memory objects into a compact, deterministic byte format.
///
/// # Examples
///
/// ```
/// use libvctrl::codec::{BinaryEncoder, BinaryDecoder};
/// use libvctrl::{Blob, Encoder, Decoder};
///
/// let blob = Blob::new(b"data".to_vec());
/// let bytes = BinaryEncoder.encode_blob(&blob).unwrap();
/// let decoded = BinaryDecoder.decode_blob(&bytes).unwrap();
/// assert_eq!(decoded, blob);
/// ```
pub use reference::codec;

/// Re-exports of object builder modules.
///
/// # Purpose
/// Provides fluent APIs like [`CommitBuilder`] to ergonomically assemble
/// complex objects step-by-step.
///
/// # Examples
///
/// ```
/// use libvctrl::object::BlobBuilder;
///
/// let blob = BlobBuilder::new()
///     .with_data(b"hello".to_vec())
///     .build();
/// assert_eq!(blob.size(), 5);
/// ```
pub use reference::object;

/// Re-exports of in-memory storage modules.
///
/// # Purpose
/// Provides concrete [`ObjectStore`] and [`RefStore`] implementations,
/// such as [`MemoryStore`], for persisting data in RAM.
///
/// # Examples
///
/// ```
/// use libvctrl::store::MemoryStore;
/// use libvctrl::{Hash, ObjectStore};
///
/// let mut store = MemoryStore::new();
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// store.put(&hash, b"data").unwrap();
/// assert!(store.exists(&hash).unwrap());
/// ```
pub use reference::store;

/// Re-exports of validation utility modules.
///
/// # Purpose
/// Provides helper functions to validate raw inputs (like names and hashes)
/// before they are turned into strongly-typed objects.
///
/// # Examples
///
/// ```
/// use libvctrl::validate::name::validate_name;
/// assert!(validate_name("valid_name").is_ok());
/// assert!(validate_name("../invalid").is_err());
/// ```
pub use reference::validate;

/// Re-export of the binary decoder struct.
///
/// # Purpose
/// Implements the [`Decoder`] trait to parse the binary representation
/// generated by [`BinaryEncoder`] back into in-memory objects.
///
/// # Design Rationale
/// It is re-exported at the root to provide immediate access to the standard
/// wire format decoder without requiring users to navigate deep module paths.
///
/// # Examples
///
/// ```
/// use libvctrl::{BinaryDecoder, BinaryEncoder, Blob, Decoder, Encoder};
///
/// let blob = Blob::new(b"data".to_vec());
/// let bytes = BinaryEncoder.encode_blob(&blob).unwrap();
/// let decoded = BinaryDecoder.decode_blob(&bytes).unwrap();
/// assert_eq!(decoded, blob);
/// ```
pub use reference::codec::BinaryDecoder;

/// Re-export of the binary encoder struct.
///
/// # Purpose
/// Implements the [`Encoder`] trait to convert in-memory objects into a
/// deterministic binary representation suitable for storage.
///
/// # Examples
///
/// ```
/// use libvctrl::{BinaryEncoder, Blob, Encoder};
///
/// let encoder = BinaryEncoder;
/// let blob = Blob::new(vec![1, 2, 3]);
/// assert!(encoder.encode_blob(&blob).is_ok());
/// ```
pub use reference::codec::BinaryEncoder;

/// Re-export of the SHA-512 hasher adapter.
///
/// # Purpose
/// Bridges the pure-Rust `libvctrl_sha512` crate with the core [`Hasher`]
/// trait, allowing it to be used transparently by the VCS to generate
/// content-addressable identifiers.
///
/// # Examples
///
/// ```
/// use libvctrl::{Hasher, Sha512Hasher};
///
/// let hasher = Sha512Hasher;
/// let hash = hasher.hash(b"data");
/// assert_eq!(hash.as_bytes().len(), 64);
/// ```
pub use reference::hash::Sha512Hasher;

/// Re-export of the `Blob` builder.
///
/// # Purpose
/// Provides a fluent interface for assembling a [`Blob`]'s data before
/// finalizing it into an immutable object.
///
/// # Examples
///
/// ```
/// use libvctrl::BlobBuilder;
///
/// let blob = BlobBuilder::default().build();
/// assert!(blob.is_empty());
/// ```
pub use reference::object::BlobBuilder;

/// Re-export of the `Commit` builder.
///
/// # Purpose
/// Solves the "telescoping constructor" problem for [`Commit`] objects by
/// allowing step-by-step configuration of required and optional fields.
///
/// # Examples
///
/// ```
/// use libvctrl::{CommitBuilder, Hash, UserID};
///
/// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let user = UserID::new("A".to_string(), "a@a.com".to_string()).unwrap();
///
/// let commit = CommitBuilder::new()
///     .tree(tree)
///     .author(user.clone())
///     .committer(user)
///     .message("msg")
///     .build()
///     .unwrap();
///
/// assert_eq!(commit.parents().len(), 0);
/// ```
pub use reference::object::CommitBuilder;

/// Re-export of the `Tag` builder.
///
/// # Purpose
/// Provides a fluent API for constructing [`Tag`] objects, handling the
/// combination of required (`name`, `target`) and optional (`tagger`, `meta`)
/// fields.
///
/// # Examples
///
/// ```
/// use libvctrl::{Hash, TagBuilder};
///
/// let target = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let tag = TagBuilder::new()
///     .name("v2.0")
///     .target(target)
///     .build()
///     .unwrap();
///
/// assert_eq!(tag.name(), "v2.0");
/// ```
pub use reference::object::TagBuilder;

/// Re-export of the `Tree` builder.
///
/// # Purpose
/// Accumulates [`TreeEntry`] objects and finalizes them into an immutable
/// [`Tree`], enforcing structural invariants like sorted entries.
///
/// # Examples
///
/// ```
/// use libvctrl::{EntryKind, Hash, TreeBuilder};
///
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let tree = TreeBuilder::new()
///     .add_entry("a.txt".to_string(), EntryKind::Blob, hash)
///     .unwrap()
///     .build()
///     .unwrap();
///
/// assert_eq!(tree.entries().len(), 1);
/// ```
pub use reference::object::TreeBuilder;

/// Re-export of the `TreeEntry` builder.
///
/// # Purpose
/// Assembles a tree entry's data (name, kind, hash) before finalizing it,
/// deferring name validation to the `build()` step.
///
/// # Examples
///
/// ```
/// use libvctrl::{EntryKind, Hash, TreeEntryBuilder};
///
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let entry = TreeEntryBuilder::new("file.txt".to_string(), EntryKind::Blob, hash)
///     .build()
///     .unwrap();
///
/// assert_eq!(entry.name(), "file.txt");
/// ```
pub use reference::object::TreeEntryBuilder;

/// Re-export of the in-memory reference store.
///
/// # Purpose
/// Maps human-readable reference names (e.g., "HEAD") to cryptographic
/// [`struct@Hash`]es in RAM. Ideal for testing and ephemeral operations.
///
/// # Examples
///
/// ```
/// use libvctrl::{Hash, MemoryRefStore, RefStore};
///
/// let mut store = MemoryRefStore::new();
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// store.set_ref("HEAD", &hash).unwrap();
/// assert!(store.get_ref("HEAD").is_ok());
/// ```
pub use reference::store::MemoryRefStore;

/// Re-export of the in-memory object store.
///
/// # Purpose
/// Stores raw, serialized version control objects in a `HashMap` residing in
/// RAM, addressable by their [`struct@Hash`].
///
/// # Examples
///
/// ```
/// use libvctrl::{Hash, MemoryStore, ObjectStore};
/// use std::io::Read;
///
/// let mut store = MemoryStore::new();
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// store.put(&hash, b"data").unwrap();
///
/// let mut buf = Vec::new();
/// store.get(&hash).unwrap().read_to_end(&mut buf).unwrap();
/// assert_eq!(buf, b"data");
/// ```
pub use reference::store::MemoryStore;

/// Re-export of the hash validation utility.
///
/// # Purpose
/// Ensures that byte slices intended to represent hashes meet the strict
/// length requirements (64 bytes) before being converted to the [`struct@Hash`] type.
///
/// # Examples
///
/// ```
/// use libvctrl::validate_hash_bytes;
/// use libvctrl::handler::HASH_LENGTH;
///
/// let valid_bytes = [0u8; HASH_LENGTH];
/// assert!(validate_hash_bytes(&valid_bytes).is_ok());
/// ```
pub use reference::validate::hash::validate_hash_bytes;

/// Re-export of the name validation utility.
///
/// # Purpose
/// Acts as a gatekeeper for strings used as identifiers (e.g., branches, tags),
/// ensuring they are non-empty, within length limits, and free of path
/// traversal characters (`/`, `.`, `..`).
///
/// # Examples
///
/// ```
/// use libvctrl::validate_name;
///
/// assert!(validate_name("valid_name").is_ok());
/// assert!(validate_name("../invalid").is_err());
/// ```
pub use reference::validate::name::validate_name;
