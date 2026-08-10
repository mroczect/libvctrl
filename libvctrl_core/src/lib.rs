//! # `libvctrl_core` – Batteries-Included Implementations for `libvctrl_handler`
//!
//! **The reference implementation layer for building modular version control systems.**
//!
//! This crate provides production-ready, safe implementations of the
//! abstract contracts defined in [`libvctrl_handler`]. It is the
//! *first consumer* of those contracts, validating their design by
//! building a complete, working VCS backend stack.
//!
//! ## Why this crate exists
//!
//! - **Validation of contracts** – If a trait is too difficult to implement,
//!   the problem is caught here before downstream users encounter it.
//! - **Batteries included** – Get a working VCS core (hashing, storage,
//!   encoding, validation) in seconds, without writing boilerplate.
//! - **Quality exemplar** – All code is safe, strictly linted (`#![forbid(unsafe_code)]`,
//!   `clippy::pedantic`, `clippy::nursery`), heavily tested, and documented
//!   to serve as a model for custom backend implementations.
//!
//! ## Architecture & Modules
//!
//! The crate is structured by domain responsibility, mirroring the
//! separations in `libvctrl_handler`:
//!
//! | Module | Purpose | Key types/traits implemented |
//! |---|---|---|
//! | [`codec`] | Binary serialization/deserialization | `Encoder`, `Decoder` (via `BinaryEncoder`, `BinaryDecoder`) |
//! | [`hash`] | Cryptographic hashing | `Hasher` (via `Sha512Hasher`) |
//! | [`object`] | Builder patterns for ergonomic construction | `BlobBuilder`, `CommitBuilder`, `TagBuilder`, `TreeBuilder` |
//! | [`store`] | Ephemeral in-memory storage | `ObjectStore` (via `MemoryStore`), `RefStore` (via `MemoryRefStore`) |
//! | [`validate`] | Security and structure validation | `validate_name`, `validate_hash_bytes` |
//!
//! ## Key Features
//!
//! - **Streaming object reads** – [`MemoryStore::get`] returns
//!   [`Box<dyn std::io::Read>`][std::io::Read] for zero-copy, lazy access,
//!   aligning with the `libvctrl_handler` v4.0.0 streaming contracts.
//! - **Iterator-based ref listing** – [`MemoryRefStore::list_refs`] returns
//!   a lazy iterator, enabling efficient handling of millions of references.
//! - **Full POSIX tree fidelity** – Encoder/decoder support all five
//!   [`EntryKind`][libvctrl_handler::EntryKind] variants:
//!   `Blob`, `Executable`, `Symlink`, `Tree`, `Submodule`.
//! - **Robust binary format** – Compact, little-endian binary encoding
//!   with versioning, bounds checks, and `DoS` protection.
//! - **Defensive validation** – [`validate_name`][crate::validate::name::validate_name]
//!   prevents path traversal attacks; [`validate_hash_bytes`][crate::validate::hash::validate_hash_bytes]
//!   enforces strict hash integrity.
//! - **Thread-safe and safe** – `#![forbid(unsafe_code)]` guarantees no
//!   undefined behavior; all types are `Send + Sync`.
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! libvctrl_core = "1.1"
//! ```
//!
//! Then integrate hashing, encoding, and storage in one go:
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
//! let hash = hasher.hash(&bytes);
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
/// Provides the [`BinaryEncoder`](crate::codec::BinaryEncoder) and
/// [`BinaryDecoder`](crate::codec::BinaryDecoder) which translate in-memory
/// objects into a compact, deterministic byte format.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Blob, Encoder, Decoder};
/// use libvctrl_core::codec::{BinaryEncoder, BinaryDecoder};
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
/// Provides concrete [`Hasher`](libvctrl_handler::Hasher) implementations,
/// such as [`Sha512Hasher`](crate::hash::Sha512Hasher), for content addressing.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::Hasher;
/// use libvctrl_core::hash::Sha512Hasher;
///
/// let hasher = Sha512Hasher;
/// let hash = hasher.hash(b"data");
/// assert_eq!(hash.as_bytes().len(), 64);
/// ```
pub mod hash;

/// Builder patterns for constructing version control objects.
///
/// # Purpose
/// Provides fluent APIs like [`CommitBuilder`](crate::object::CommitBuilder)
/// to ergonomically assemble complex objects step-by-step.
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
/// Provides concrete [`ObjectStore`](libvctrl_handler::ObjectStore) and
/// [`RefStore`](libvctrl_handler::RefStore) implementations, such as
/// [`MemoryStore`](crate::store::MemoryStore), for persisting data in RAM.
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
/// Provides helper functions to validate raw inputs (like names and hashes)
/// before they are turned into strongly-typed objects, preventing path
/// traversal and resource exhaustion.
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
