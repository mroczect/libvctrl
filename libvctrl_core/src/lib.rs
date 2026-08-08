//! # `libvctrl_core` – Reference Implementations
//!
//! This crate provides **concrete, minimal, and correct** implementations
//! for every trait defined in [`libvctrl_handler`].
//!
//! It is not intended to be the fastest or most featureful backend.
//! Its purpose is to prove that the contracts can be fulfilled and to
//! serve as a readable reference for anyone building their own
//! version control primitives.
//!
//! ## Crate architecture
//!
//! The crate is organised into five modules, each tackling one aspect of
//! the version control model:
//!
//! | Module | Purpose |
//! |---|---|
//! | [`validate`] | Reusable validation helpers for names and hashes |
//! | [`store`] | In‑memory object and reference stores |
//! | [`hash`] | A SHA‑512 hasher based on `libvctrl_sha512` |
//! | [`object`] | Builder patterns for the four core object types |
//! | [`codec`] | Binary encoder/decoder with a deterministic format |
//!
//! ## Philosophy
//!
//! - **Reference, not production** – these implementations are correct
//!   and safe, but not optimised for throughput or concurrency. Use them
//!   for testing, prototyping, and as a starting point for your own
//!   backends.
//! - **No unsafe code** – the entire crate is `#![forbid(unsafe_code)]`.
//!   All memory safety guarantees are upheld by the Rust compiler.
//! - **Panic‑free** – all public APIs return `Result` and never panic on
//!   invalid input. The only potential panics are programmer errors
//!   (e.g., calling a builder’s `build()` without setting required fields),
//!   which are caught at development time.
//! - **Contracts first** – every implementation strictly adheres to the
//!   trait contracts defined in `libvctrl_handler`. If a trait says a
//!   name must not be empty, this crate enforces that.
//!
//! ## Module details
//!
//! ### `validate`
//! Provides [`validate_hash_bytes`] and [`validate_name`]. These are
//! lightweight functions that check raw input before constructing
//! domain types. They are the **single source of truth** for basic
//! constraints like hash length and valid names.
//!
//! ### `store`
//! Contains [`MemoryStore`] (an in‑memory [`ObjectStore`]) and
//! [`MemoryRefStore`] (an in‑memory [`RefStore`]). Both are backed by
//! `HashMap` and are suitable for tests and prototyping.
//!
//! ### `hash`
//! Houses [`Sha512Hasher`], a stateless [`Hasher`] that delegates to
//! the audited `libvctrl_sha512` crate. It produces 64‑byte SHA‑512
//! digests.
//!
//! ### `object`
//! Builders for [`Blob`], [`Tree`], [`Commit`], and [`Tag`]. Each
//! builder provides a fluent API for setting fields incrementally and
//! validating required fields at build time.
//!
//! ### `codec`
//! The [`BinaryEncoder`] and [`BinaryDecoder`] define a simple,
//! deterministic binary format for all object types. The decoder
//! includes DoS‑prevention limits (blob size, tree entries, message
//! length) sourced from `libvctrl_handler::constants`.
//!
//! ## Quick example
//!
//! ```rust
//! use libvctrl_core::store::MemoryStore;
//! use libvctrl_handler::{ObjectStore, Hash, HASH_LENGTH};
//!
//! let mut store = MemoryStore::new();
//! let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
//! store.put(&hash, b"Hello").unwrap();
//! assert_eq!(store.get(&hash).unwrap(), b"Hello");
//! ```
//!
//! A more complete example using all modules:
//!
//! ```rust
//! use libvctrl_core::hash::Sha512Hasher;
//! use libvctrl_core::object::{CommitBuilder, TreeBuilder};
//! use libvctrl_core::codec::{BinaryEncoder, BinaryDecoder};
//! use libvctrl_handler::*;
//!
//! // 1. Hash some content
//! let hasher = Sha512Hasher;
//! let blob_hash = hasher.hash(b"file content");
//!
//! // 2. Build a tree
//! let tree = TreeBuilder::new()
//!     .add_entry("file.txt".into(), EntryKind::Blob, blob_hash)
//!     .unwrap()
//!     .build()
//!     .unwrap();
//!
//! // 3. Hash the tree
//! let tree_bytes = BinaryEncoder.encode_tree(&tree).unwrap();
//! let tree_hash = hasher.hash(&tree_bytes);
//!
//! // 4. Build a commit
//! let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
//! let commit = CommitBuilder::new()
//!     .tree(tree_hash)
//!     .author(author.clone())
//!     .committer(author)
//!     .message("First commit")
//!     .build()
//!     .unwrap();
//!
//! // 5. Round‑trip through codec
//! let commit_bytes = BinaryEncoder.encode_commit(&commit).unwrap();
//! let commit2 = BinaryDecoder.decode_commit(&commit_bytes).unwrap();
//! assert_eq!(commit, commit2);
//! ```
//!
//! ## Relationship to other crates
//!
//! - `libvctrl_handler` – defines the traits and types this crate implements.
//! - `libvctrl_sha512` – provides the SHA‑512 algorithm used by `Sha512Hasher`.
//! - `libvctrl_plumbing` – (future) will use these implementations to build
//!   atomic version control operations.
//! - `libvctrl_porcelain` – (future) will provide high‑level user‑friendly
//!   commands built on top of plumbing.
//!
//! ## License
//!
//! MIT – see the repository root for details.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod codec;
/// Cryptographic hash function implementations.
pub mod hash;
/// Builders for core object types (Blob, Tree, Commit, Tag).
pub mod object;
/// In‑memory reference implementations of the storage traits.
pub mod store;
/// Common validation utilities shared across modules.
pub mod validate;
