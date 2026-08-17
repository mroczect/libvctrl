//! # libvctrl
//!
//! A unified facade for the libvctrl ecosystem.
//!
//! This crate aggregates the foundational crates of the version control
//! system into a single, coherent namespace. It re-exports all core types,
//! traits, constants, validation functions, and reference implementations
//! from:
//!
//! - [`libvctrl_handler`](https://docs.rs/libvctrl_handler) — abstract
//!   contracts, immutable data types, and system limits.
//! - [`libvctrl_core`](https://docs.rs/libvctrl_core) — production-ready
//!   reference implementations: binary codec, SHA-512 hasher, builders, and
//!   in-memory stores.
//! - [`libvctrl_sha512`](https://docs.rs/libvctrl_sha512) — zero-dependency
//!   cryptographic primitives.
//!
//! By re-exporting these crates under one roof, `libvctrl` allows downstream
//! applications to bootstrap a complete version control system without
//! manually stitching together multiple dependencies. It also serves as the
//! public API surface for the main binary crate.
//!
//! ## Architecture
//!
//! The crate exposes three top-level namespaces:
//!
//! - [`handler`](crate::handler) — the original `libvctrl_handler` crate.
//! - [`reference`](crate::reference) — the `libvctrl_core` reference
//!   implementation crate.
//! - [`crypto`](crate::crypto) — the `libvctrl_sha512` crate.
//!
//! In addition, the most commonly used items are re-exported directly at the
//! crate root for ergonomic access.
//!
//! ### Handler re-exports
//!
//! Core contracts and types:
//!
//! - Traits: [`Encoder`](crate::Encoder), [`Decoder`](crate::Decoder),
//!   [`Hasher`](crate::Hasher), [`ObjectStore`](crate::ObjectStore),
//!   [`RefStore`](crate::RefStore), [`Signer`](crate::Signer),
//!   [`Verifier`](crate::Verifier), [`Transport`](crate::Transport).
//! - Types: [`Blob`](crate::Blob), [`Tree`](crate::Tree),
//!   [`TreeEntry`](crate::TreeEntry), [`Commit`](crate::Commit),
//!   [`CommitMeta`](crate::CommitMeta), [`Tag`](crate::Tag),
//!   [`Hash`](crate::Hash), [`UserID`](crate::UserID),
//!   [`EntryKind`](crate::EntryKind).
//! - Error type: [`VctrlError`](crate::VctrlError).
//!
//! System limits and validation:
//!
//! - Constants such as [`HASH_LENGTH`](crate::HASH_LENGTH),
//!   [`MAX_BLOB_SIZE`](crate::MAX_BLOB_SIZE),
//!   [`MAX_MESSAGE_LENGTH`](crate::MAX_MESSAGE_LENGTH),
//!   [`MAX_NAME_LENGTH`](crate::MAX_NAME_LENGTH),
//!   [`MAX_PARENT_COUNT`](crate::MAX_PARENT_COUNT), and
//!   [`MAX_TREE_ENTRIES`](crate::MAX_TREE_ENTRIES).
//! - Validation functions:
//!   [`validate_hash_bytes`](crate::validate_hash_bytes),
//!   [`validate_name`](crate::validate_name),
//!   [`validate_ref_name`](crate::validate_ref_name), and
//!   [`validate_tree_entry_name`](crate::validate_tree_entry_name).
//!
//! ### Core re-exports
//!
//! Reference implementations:
//!
//! - Codec: [`BinaryEncoder`](crate::BinaryEncoder) and
//!   [`BinaryDecoder`](crate::BinaryDecoder) for deterministic binary
//!   serialization.
//! - Hasher: [`Sha512Hasher`](crate::Sha512Hasher) for content addressing.
//! - Builders: [`BlobBuilder`](crate::BlobBuilder),
//!   [`CommitBuilder`](crate::CommitBuilder),
//!   [`TagBuilder`](crate::TagBuilder),
//!   [`TreeBuilder`](crate::TreeBuilder), and
//!   [`TreeEntryBuilder`](crate::TreeEntryBuilder).
//! - Stores: [`MemoryStore`](crate::MemoryStore) and
//!   [`MemoryRefStore`](crate::MemoryRefStore).
//!
//! ## Why a unified facade?
//!
//! The libvctrl workspace is designed around strict separation of concerns.
//! However, end users often need a single dependency that exposes the full
//! stack. This crate provides that convenience without hiding the underlying
//! modularity. Developers can still access the original crates through the
//! `handler`, `reference`, and `crypto` namespaces.
//!
//! ## How it works
//!
//! All re-exports are compile-time aliases. There is no runtime overhead, and
//! no code is duplicated. The only cost is a slightly larger public API
//! surface.
//!
//! ## Safety and quality
//!
//! This crate inherits the strict safety guarantees of its dependencies:
//!
//! - `#![forbid(unsafe_code)]` — no unsafe code, period.
//! - Strict Clippy, rustc, and documentation lints are denied.
//! - All public items are documented and have doctests where applicable.
//!
//! ## Example
//!
//! The following example demonstrates a typical workflow: create a blob,
//! encode it, hash it, store it, and retrieve it.
//!
//! ```
//! # use libvctrl::{Blob, Encoder, Hasher, ObjectStore, BinaryEncoder, Sha512Hasher, MemoryStore};
//! # fn main() -> Result<(), libvctrl::VctrlError> {
//! let blob = Blob::new(b"my content".to_vec())?;
//!
//! // Encode the blob into deterministic bytes.
//! let mut encoded = Vec::new();
//! BinaryEncoder.encode_blob(&blob, &mut encoded)?;
//!
//! // Hash the encoded bytes to obtain a content address.
//! let hash = Sha512Hasher.hash(&mut encoded.as_slice())?;
//!
//! // Store the encoded object in memory.
//! let mut store = MemoryStore::new();
//! store.put(&hash, &encoded)?;
//!
//! // Verify the object exists.
//! assert!(store.exists(&hash)?);
//! # Ok(())
//! # }
//! ```
//!
//! Use [`handler`](crate::handler), [`reference`](crate::reference), or
//! [`crypto`](crate::crypto) if you need direct access to the underlying
//! crates.

/// Re-export of the `libvctrl_core` reference implementation crate.
///
/// This namespace contains production-ready implementations of the handler
/// contracts: binary codec, SHA-512 hasher, builders, and in-memory stores.
pub use libvctrl_core as reference;

/// Re-export of the `libvctrl_handler` contracts and types crate.
///
/// This namespace contains the abstract traits, immutable data types,
/// validation functions, and system constants that define the core VCS model.
pub use libvctrl_handler as handler;

/// Re-export of the `libvctrl_sha512` cryptographic primitives crate.
///
/// This namespace exposes zero-dependency SHA-512, HMAC-SHA512, HKDF-SHA512,
/// and optional SHA-384 implementations.
pub use libvctrl_sha512 as crypto;

/// Handler module re-exports.
///
/// These modules are re-exported for direct access to the original crate's
/// internal organization. Most users will prefer the flattened root items,
/// but these are available for advanced use cases.
pub use handler::constants;

/// Enumerations and kind discriminants.
///
/// Contains [`EntryKind`](crate::EntryKind) and any other enum types defined
/// by the handler crate.
pub use handler::enums;

/// Error types and constructors.
///
/// Contains [`VctrlError`](crate::VctrlError) and associated error variants.
pub use handler::errors;

/// Macros exported by the handler crate.
///
/// These macros assist in implementing common traits or validation logic.
pub use handler::macros;

/// Core behavior traits.
///
/// Contains the trait definitions for [`Encoder`](crate::Encoder),
/// [`Decoder`](crate::Decoder), [`Hasher`](crate::Hasher),
/// [`ObjectStore`](crate::ObjectStore), [`RefStore`](crate::RefStore),
/// [`Signer`](crate::Signer), [`Verifier`](crate::Verifier), and
/// [`Transport`](crate::Transport).
pub use handler::traits;

/// Immutable data types.
///
/// Contains the core object model: [`Blob`](crate::Blob),
/// [`Tree`](crate::Tree), [`TreeEntry`](crate::TreeEntry),
/// [`Commit`](crate::Commit), [`CommitMeta`](crate::CommitMeta),
/// [`Tag`](crate::Tag), [`Hash`](crate::Hash), [`UserID`](crate::UserID),
/// and related types.
pub use handler::types;

/// Validation helper functions.
///
/// Contains functions like [`validate_name`](crate::validate_name) and
/// [`validate_ref_name`](crate::validate_ref_name) used to enforce safety
/// invariants.
pub use handler::validation;

/// System limit constants.
///
/// Re-exports the following constants at the crate root:
///
/// - [`HASH_LENGTH`](crate::HASH_LENGTH)
/// - [`MAX_BLOB_SIZE`](crate::MAX_BLOB_SIZE)
/// - [`MAX_MESSAGE_LENGTH`](crate::MAX_MESSAGE_LENGTH)
/// - [`MAX_NAME_LENGTH`](crate::MAX_NAME_LENGTH)
/// - [`MAX_PARENT_COUNT`](crate::MAX_PARENT_COUNT)
/// - [`MAX_TREE_ENTRIES`](crate::MAX_TREE_ENTRIES)
pub use handler::{
    HASH_LENGTH, MAX_BLOB_SIZE, MAX_MESSAGE_LENGTH, MAX_NAME_LENGTH, MAX_PARENT_COUNT,
    MAX_TREE_ENTRIES,
};

/// Represents the kind of a tree entry.
///
/// This enum distinguishes blobs, executable files, symlinks, trees, and
/// submodules.
pub use handler::EntryKind;

/// Unified error type for all libvctrl operations.
///
/// All fallible operations across the ecosystem return this error type.
pub use handler::VctrlError;

/// Core behavior traits.
///
/// Re-exports the following traits at the crate root:
///
/// - [`Decoder`](crate::Decoder)
/// - [`Encoder`](crate::Encoder)
/// - [`Hasher`](crate::Hasher)
/// - [`ObjectStore`](crate::ObjectStore)
/// - [`RefStore`](crate::RefStore)
/// - [`Signer`](crate::Signer)
/// - [`Transport`](crate::Transport)
/// - [`Verifier`](crate::Verifier)
pub use handler::{Decoder, Encoder, Hasher, ObjectStore, RefStore, Signer, Transport, Verifier};

/// Immutable data types.
///
/// Re-exports the following types at the crate root:
///
/// - [`Blob`](crate::Blob)
/// - [`Commit`](crate::Commit)
/// - [`CommitMeta`](crate::CommitMeta)
/// - [`Hash`](crate::Hash)
/// - [`Tag`](crate::Tag)
/// - [`Tree`](crate::Tree)
/// - [`TreeEntry`](crate::TreeEntry)
/// - [`UserID`](crate::UserID)
pub use handler::{Blob, Commit, CommitMeta, Hash, Tag, Tree, TreeEntry, UserID};

/// Validation functions.
///
/// Re-exports the following functions at the crate root:
///
/// - [`validate_hash_bytes`](crate::validate_hash_bytes)
/// - [`validate_name`](crate::validate_name)
/// - [`validate_ref_name`](crate::validate_ref_name)
/// - [`validate_tree_entry_name`](crate::validate_tree_entry_name)
pub use handler::{
    validate_hash_bytes, validate_name, validate_ref_name, validate_tree_entry_name,
};

/// Core reference implementation re-exports.
///
/// These items provide concrete implementations of the handler contracts.
pub use reference::codec;

/// Object builders for ergonomic construction.
///
/// This module contains builder types for blobs, commits, tags, trees, and
/// tree entries.
pub use reference::object;

/// In-memory object and reference stores.
///
/// This module contains [`MemoryStore`](crate::MemoryStore) and
/// [`MemoryRefStore`](crate::MemoryRefStore).
pub use reference::store;

/// Decoder for the binary format.
///
/// This zero-sized type implements [`Decoder`](crate::Decoder) and parses
/// versioned binary payloads with strict bounds checking.
pub use reference::codec::BinaryDecoder;

/// Encoder for the binary format.
///
/// This zero-sized type implements [`Encoder`](crate::Encoder) and produces
/// deterministic, versioned binary payloads.
pub use reference::codec::BinaryEncoder;

/// SHA-512 content hasher.
///
/// This type implements [`Hasher`](crate::Hasher) and produces 64-byte
/// content addresses.
pub use reference::hash::Sha512Hasher;

/// Builder for [`Blob`] objects.
///
/// Provides a fluent API for constructing validated blobs.
pub use reference::object::BlobBuilder;

/// Builder for [`Commit`] objects.
///
/// Provides a fluent API for constructing validated commits.
pub use reference::object::CommitBuilder;

/// Builder for [`Tag`] objects.
///
/// Provides a fluent API for constructing validated tags.
pub use reference::object::TagBuilder;

/// Builder for [`Tree`] objects.
///
/// Provides a fluent API for constructing validated trees.
pub use reference::object::TreeBuilder;

/// Builder for [`TreeEntry`] objects.
///
/// Provides a fluent API for constructing validated tree entries.
pub use reference::object::TreeEntryBuilder;

/// In-memory reference store.
///
/// Implements [`RefStore`](crate::RefStore) using a `HashMap`.
pub use reference::store::MemoryRefStore;

/// In-memory object store.
///
/// Implements [`ObjectStore`](crate::ObjectStore) using a `HashMap`.
pub use reference::store::MemoryStore;
