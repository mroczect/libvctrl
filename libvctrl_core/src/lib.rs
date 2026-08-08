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
//! ## Modules
//! - [`validate`] – Reusable validation helpers (name, hash).
//! - [`store`] – In‑memory object and reference stores.
//! - [`hash`] – SHA‑512 hasher.
//! - [`object`] – Builders for the four core object types.
//! - [`codec`] – Binary encoder/decoder with a simple deterministic format.
//!
//! ## Usage
//! ```rust
//! use libvctrl_core::store::MemoryStore;
//! use libvctrl_handler::{ObjectStore, Hash, HASH_LENGTH};
//!
//! let mut store = MemoryStore::new();
//! let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
//! store.put(&hash, b"Hello").unwrap();
//! assert_eq!(store.get(&hash).unwrap(), b"Hello");
//! ```

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
