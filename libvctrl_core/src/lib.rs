//! # `libvctrl_core` -- Batteries-Included Implementations for `libvctrl_handler`
//!
//! **The reference implementation layer for building modular version control systems.**
//!
//! This crate provides production-ready, safe implementations of the
//! abstract contracts defined in [`libvctrl_handler`]. It is the
//! *first consumer* of those contracts, validating their design by
//! building a complete, working VCS backend stack.
//!
//! # Why This Crate Exists
//!
//! - **Validation of contracts**: If a trait is too difficult to implement,
//!   the problem is caught here before downstream users encounter it.
//! - **Batteries included**: Get a working VCS core (hashing, storage,
//!   encoding, validation) in seconds, without writing boilerplate.
//! - **Quality exemplar**: All code is safe, strictly linted
//!   (`#![forbid(unsafe_code)]`, `clippy::pedantic`, `clippy::nursery`),
//!   heavily tested, and documented to serve as a model for custom backend
//!   implementations.
//!
//! # Architecture and Modules
//!
//! The crate is structured by domain responsibility, mirroring the
//! separations in `libvctrl_handler`:
//!
//! | Module | Purpose | Key types/traits implemented |
//! |---|---|---|
//! | [`codec`] | Binary serialization/deserialization | `Encoder`, `Decoder` (via [`BinaryEncoder`](codec::BinaryEncoder), [`BinaryDecoder`](codec::BinaryDecoder)) |
//! | [`hash`] | Cryptographic hashing | `Hasher` (via [`Sha512Hasher`](hash::Sha512Hasher)) |
//! | [`object`] | Builder patterns for ergonomic construction | [`BlobBuilder`](object::BlobBuilder), [`CommitBuilder`](object::CommitBuilder), [`TagBuilder`](object::TagBuilder), [`TreeBuilder`](object::TreeBuilder) |
//! | [`store`] | Ephemeral in-memory storage | `ObjectStore` (via [`MemoryStore`](store::MemoryStore)), `RefStore` (via [`MemoryRefStore`](store::MemoryRefStore)) |
//! | [`validate`] | Security and structure validation | [`validate_name`](validate::name::validate_name), [`validate_hash_bytes`](validate::hash::validate_hash_bytes) |
//!
//! # Design Philosophy
//!
//! `libvctrl_core` is built on several foundational principles:
//!
//! 1. **Safety first**: The crate is `#![forbid(unsafe_code)]`. No unsafe
//!    code is allowed anywhere, eliminating the possibility of undefined
//!    behavior from this crate's own logic.
//! 2. **Streaming by default**: Object reads return
//!    [`Box<dyn std::io::Read>`][std::io::Read] instead of a full
//!    [`Vec<u8>`], allowing callers to process large objects incrementally
//!    without large contiguous allocations.
//! 3. **Defensive validation**: Inputs are validated at the boundaries.
//!    The [`validate`] module prevents path traversal and resource
//!    exhaustion before data reaches the strongly-typed object layer.
//! 4. **Deterministic behavior**: Encoders, decoders, and stores are
//!    deterministic by design. Trees must be sorted, maps are sorted before
//!    iteration, and the binary format is fully specified.
//! 5. **Zero-cost abstraction**: The crate leverages Rust's type system and
//!    the contracts from `libvctrl_handler` without adding overhead. The
//!    hasher is a zero-sized type; builders consume `self` and move data
//!    without cloning.
//!
//! # Key Features
//!
//! - **Streaming object reads**: [`MemoryStore::get`] returns
//!   [`Box<dyn std::io::Read>`][std::io::Read] for zero-copy, lazy access,
//!   aligning with the `libvctrl_handler` v4.0.0 streaming contracts.
//! - **Iterator-based ref listing**: [`MemoryRefStore::list_refs`] returns
//!   a lazy iterator, enabling efficient handling of millions of references.
//! - **Full POSIX tree fidelity**: Encoder/decoder support all five
//!   [`EntryKind`][libvctrl_handler::EntryKind] variants:
//!   `Blob`, `Executable`, `Symlink`, `Tree`, `Submodule`.
//! - **Robust binary format**: Compact, little-endian binary encoding
//!   with versioning, bounds checks, and DoS protection.
//! - **Defensive validation**: [`validate_name`](crate::validate::name::validate_name)
//!   prevents path traversal attacks;
//!   [`validate_hash_bytes`](crate::validate::hash::validate_hash_bytes)
//!   enforces strict hash integrity.
//! - **Thread-safe and safe**: `#![forbid(unsafe_code)]` guarantees no
//!   undefined behavior; all types are `Send + Sync` where applicable.
//!
//! # Relationship to `libvctrl_handler`
//!
//! This crate consumes the traits and data types defined by
//! [`libvctrl_handler`]. It does not redefine those contracts; instead, it
//! provides concrete implementations that can be used directly or swapped
//! out for custom backends. The dependency graph is intentionally one-way:
//! `libvctrl_core` depends on `libvctrl_handler`, never the reverse.
//!
//! # How to Use This Crate
//!
//! ## Adding the dependency
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! libvctrl_core = "2.0"
//! libvctrl_handler = "4.4"
//! ```
//!
//! ## Quick start
//!
//! Integrate hashing, encoding, and storage in one go:
//!
//! ```
//! use libvctrl_handler::{Blob, Encoder, Hasher, ObjectStore};
//! use libvctrl_core::codec::BinaryEncoder;
//! use libvctrl_core::hash::Sha512Hasher;
//! use libvctrl_core::store::MemoryStore;
//! use std::io::Read;
//!
//! // 1. Create content
//! let blob = Blob::new(b"my content".to_vec());
//!
//! // 2. Encode to deterministic bytes
//! let encoder = BinaryEncoder;
//! let bytes = encoder.encode_blob(&blob).unwrap();
//!
//! // 3. Hash the bytes to get a content address
//! let hasher = Sha512Hasher;
//! let hash = hasher.hash(&bytes).unwrap();
//!
//! // 4. Store the encoded bytes in memory
//! let mut store = MemoryStore::new();
//! store.put(&hash, &bytes).unwrap();
//!
//! // 5. Read back via streaming interface
//! let mut reader = store.get(&hash).unwrap();
//! let mut buf = Vec::new();
//! reader.read_to_end(&mut buf).unwrap();
//! assert_eq!(buf, bytes);
//! ```
//!
//! ## Using builders
//!
//! Construct a commit using the fluent builder API:
//!
//! ```
//! use libvctrl_core::object::CommitBuilder;
//! use libvctrl_handler::{Hash, UserID};
//!
//! let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
//! let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
//! let committer = UserID::new("Bob".into(), "bob@example.com".into()).unwrap();
//!
//! let commit = CommitBuilder::new()
//!     .tree(tree)
//!     .author(author)
//!     .committer(committer)
//!     .message("Initial commit")
//!     .build()
//!     .unwrap();
//!
//! assert_eq!(commit.message(), "Initial commit");
//! ```
//!
//! ## Round-trip encoding and decoding
//!
//! ```
//! use libvctrl_handler::{Blob, Decoder, Encoder};
//! use libvctrl_core::codec::{BinaryDecoder, BinaryEncoder};
//!
//! let blob = Blob::new(b"round trip".to_vec());
//! let bytes = BinaryEncoder.encode_blob(&blob).unwrap();
//! let decoded = BinaryDecoder.decode_blob(&bytes).unwrap();
//! assert_eq!(blob, decoded);
//! ```
//!
//! # Internal Mechanism
//!
//! The crate is organized as a set of thin adapters and implementations:
//!
//! - [`BinaryEncoder`](codec::BinaryEncoder) and
//!   [`BinaryDecoder`](codec::BinaryDecoder) translate between the
//!   strongly-typed objects of `libvctrl_handler` and a compact binary
//!   format. The format is versioned to allow future evolution.
//! - [`Sha512Hasher`](hash::Sha512Hasher) delegates to the
//!   `libvctrl_sha512` crate, which provides a pure-Rust implementation of
//!   SHA-512. The adapter is a zero-sized type that converts the 64-byte
//!   digest into a [`libvctrl_handler::Hash`].
//! - [`MemoryStore`](store::MemoryStore) and
//!   [`MemoryRefStore`](store::MemoryRefStore) implement the storage traits
//!   using [`std::collections::HashMap`], providing average O(1) operations.
//! - Builders in the [`object`] module accumulate fields and delegate final
//!   validation to the constructors of the corresponding handler types.
//! - The [`validate`] module centralizes security and structural checks so
//!   they can be reused across the crate.
//!
//! # Safety and Lints
//!
//! The crate is compiled with the strictest Rust lint levels:
//!
//! - `#![forbid(unsafe_code)]`: No unsafe code is allowed.
//! - `#![deny(clippy::all)]`, `#![deny(clippy::pedantic)]`,
//!   `#![deny(clippy::nursery)]`: All Clippy lints are treated as hard
//!   errors, ensuring a high quality bar.
//! - `#![deny(missing_docs)]`: Every public item must have documentation,
//!   which is why this crate is thoroughly documented.
//! - `#![deny(rust_2018_idioms)]`, `#![deny(unreachable_pub)]`,
//!   `#![deny(unused_qualifications)]`: Additional idiomatic Rust checks.
//!
//! These lint settings force contributors to write clear, maintainable code
//! and prevent accidental regressions in API quality.

#![forbid(unsafe_code)]
#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    unused_crate_dependencies,
    unused_qualifications
)]
#[cfg(test)]
use proptest as _;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Binary serialization and deserialization implementations.
///
/// # Purpose
///
/// Provides the [`BinaryEncoder`](crate::codec::BinaryEncoder) and
/// [`BinaryDecoder`](crate::codec::BinaryDecoder), which translate in-memory
/// objects into a compact, deterministic byte format and back.
///
/// # Design Rationale
///
/// The codec is isolated in its own module to encapsulate all wire-format
/// concerns. The format is versioned and uses little-endian integers with
/// length-prefixed fields. This design supports future format evolution and
/// enables efficient parsing without delimiter scanning.
///
/// # Internal Mechanism
///
/// The encoder pre-allocates output buffers based on estimated object size.
/// The decoder performs strict bounds checking on every slice access,
/// returning [`VctrlError::CorruptedData`](libvctrl_handler::VctrlError::CorruptedData)
/// for malformed or truncated inputs instead of panicking.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Blob, Decoder, Encoder};
/// use libvctrl_core::codec::{BinaryDecoder, BinaryEncoder};
///
/// let blob = Blob::new(b"data".to_vec());
/// let bytes = BinaryEncoder.encode_blob(&blob).unwrap();
/// let decoded = BinaryDecoder.decode_blob(&bytes).unwrap();
/// assert_eq!(decoded, blob);
/// ```
pub mod codec;

/// Cryptographic hashing implementations.
///
/// # Purpose
///
/// Provides concrete [`Hasher`](libvctrl_handler::Hasher) implementations,
/// such as [`Sha512Hasher`](crate::hash::Sha512Hasher), for content
/// addressing.
///
/// # Design Rationale
///
/// Hashing is isolated so alternative algorithms can be added or swapped
/// without touching other modules. The current implementation delegates to
/// the audited `libvctrl_sha512` crate.
///
/// # Internal Mechanism
///
/// The hasher is a zero-sized type. It calls the one-shot SHA-512 function
/// from `libvctrl_sha512`, wraps the resulting 64-byte digest in a
/// [`libvctrl_handler::Hash`], and returns it. The conversion is infallible
/// because SHA-512 always produces exactly 64 bytes.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::Hasher;
/// use libvctrl_core::hash::Sha512Hasher;
///
/// let hasher = Sha512Hasher;
/// let hash = hasher.hash(b"data").unwrap();
/// assert_eq!(hash.as_bytes().len(), 64);
/// ```
pub mod hash;

/// Builder patterns for constructing version control objects.
///
/// # Purpose
///
/// Provides fluent APIs like [`CommitBuilder`](crate::object::CommitBuilder)
/// to ergonomically assemble complex objects step-by-step.
///
/// # Design Rationale
///
/// VCS objects such as commits and tags have many fields, some required and
/// some optional. Builders avoid the telescoping constructor problem by
/// accumulating state and validating at a single `build()` call. They
/// consume `self` and move data, eliminating unnecessary clones.
///
/// # Internal Mechanism
///
/// Each builder stores intermediate fields in `Option` or `Vec` wrappers.
/// The `build()` method consumes the builder, checks for missing required
/// fields, and delegates to the corresponding constructor in
/// `libvctrl_handler`.
///
/// # Examples
///
/// ```
/// use libvctrl_core::object::BlobBuilder;
///
/// let blob = BlobBuilder::new()
///     .with_data(b"hello".to_vec())
///     .build();
/// assert_eq!(blob.size(), 5);
/// ```
pub mod object;

/// Storage backend implementations.
///
/// # Purpose
///
/// Provides concrete [`ObjectStore`](libvctrl_handler::ObjectStore) and
/// [`RefStore`](libvctrl_handler::RefStore) implementations, such as
/// [`MemoryStore`](crate::store::MemoryStore), for persisting data in RAM.
///
/// # Design Rationale
///
/// Storage is separated into object storage and reference storage, mirroring
/// the design of persistent version control systems. In-memory backends are
/// ideal for tests and ephemeral sessions because they are fast and require
/// no disk I/O.
///
/// # Internal Mechanism
///
/// [`MemoryStore`](crate::store::MemoryStore) uses a [`HashMap`] keyed by
/// [`libvctrl_handler::Hash`]. Reads return a streaming
/// [`Box<dyn std::io::Read>`][std::io::Read] backed by a cloned buffer and
/// a cursor. [`MemoryRefStore`](crate::store::MemoryRefStore) maps names to
/// hashes and sorts keys before iteration to ensure deterministic output.
///
/// # Examples
///
/// ```
/// use libvctrl_core::store::MemoryStore;
/// use libvctrl_handler::{Hash, ObjectStore};
/// use std::io::Read;
///
/// let mut store = MemoryStore::new();
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// store.put(&hash, b"data").unwrap();
///
/// let mut reader = store.get(&hash).unwrap();
/// let mut buf = Vec::new();
/// reader.read_to_end(&mut buf).unwrap();
/// assert_eq!(buf, b"data");
/// ```
pub mod store;

/// Validation utilities for structural integrity and security.
///
/// # Purpose
///
/// Provides helper functions to validate raw inputs (like names and hashes)
/// before they are turned into strongly-typed objects, preventing path
/// traversal and resource exhaustion.
///
/// # Design Rationale
///
/// Validation is centralized here to keep the data types in
/// `libvctrl_handler` pure and to enforce consistent rules across all
/// callers. The functions are designed to fail fast with descriptive
/// errors.
///
/// # Internal Mechanism
///
/// The module is split into [`name`](crate::validate::name) and
/// [`hash`](crate::validate::hash) submodules. Name validation checks
/// emptiness, length, and path traversal patterns. Hash validation checks
/// exact length against
/// [`HASH_LENGTH`](libvctrl_handler::HASH_LENGTH).
///
/// # Examples
///
/// ```
/// use libvctrl_core::validate::name::validate_name;
///
/// assert!(validate_name("valid_name").is_ok());
/// assert!(validate_name("../invalid").is_err());
/// ```
pub mod validate;
