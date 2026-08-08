#![doc = include_str!("../README.md")]

//! # Using `libvctrl_handler` – The Unshakeable Contract
//!
//! This crate **only** defines the fundamental traits, types, errors, and constants
//! for building a version control system. **No implementations are allowed here.**
//!
//! It is the single source of truth for the entire `libvctrl` ecosystem.
//! Every other component must depend on this crate and must never redefine
//! these fundamental contracts.
//!
//! ## Quick start
//!
//! All public items are re‑exported at the crate root, so you can bring everything
//! into scope with a single `use` statement:
//!
//! ```rust
//! use libvctrl_handler::*;
//!
//! // Build a 64‑byte hash from known bytes
//! let hash = Hash::from_bytes(&[0xAB; HASH_LENGTH]).unwrap();
//!
//! // Create a validated tree entry
//! let entry = TreeEntry::new("README.md".into(), EntryKind::Blob, hash).unwrap();
//!
//! // Build a tree (directory listing)
//! let tree = Tree::new(vec![entry]).unwrap();
//!
//! // Define a user identity
//! let author = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
//!
//! // Create a commit (initial commit, no parents)
//! let commit = Commit::new(hash, vec![], author.clone(), author, "Initial import".into());
//!
//! // Attach an annotated tag
//! let tag = Tag::new("v0.1.0".into(), hash, None, "Pre‑release".into()).unwrap();
//!
//! // Use the error type
//! let err = VctrlError::Other("custom error".into());
//! // Or with the convenience macro
//! let err_macro = vctrl_error_other!("problem at {}", 42);
//! ```
//!
//! ## Crate architecture
//!
//! The crate is organised into six public modules:
//!
//! | Module | Purpose |
//! |---|---|
//! | [`constants`] | Global invariants – hash length, name limits, DoS‑prevention bounds |
//! | [`enums`] | Shared enumeration types (`EntryKind`) |
//! | [`errors`] | Unified error type (`VctrlError`) |
//! | [`macros`] | Convenience macros (`vctrl_error_other!`) |
//! | [`traits`] | Core abstractions (`ObjectStore`, `RefStore`, `Hasher`, …) |
//! | [`types`] | Fundamental data types (`Hash`, `Blob`, `Tree`, `Commit`, …) |
//!
//! Each module contains extensive documentation, pre‑ and post‑conditions, and
//! implementation notes. Refer to the module‑level docs for details.
//!
//! ## Philosophy
//!
//! - **Mechanism, not policy** – no assumptions about branches, workflows, or defaults.
//! - **Unbounded flexibility, high discipline** – everything is generic and replaceable,
//!   but every input is strictly validated.
//! - **This crate is the constitution** – all fundamental traits, types, and errors
//!   live exclusively here.
//!
//! ## Implementing the traits
//!
//! While `libvctrl_handler` provides **no** implementations, the companion crate
//! [`libvctrl_core`] offers a complete, minimal reference implementation of every
//! trait. You can study that code to see how the contracts are fulfilled.
//!
//! As a quick illustration, here is a skeleton of an in‑memory object store:
//!
//! ```rust
//! # use std::collections::HashMap;
//! use libvctrl_handler::*;
//!
//! struct MemStore(HashMap<Hash, Vec<u8>>);
//!
//! impl ObjectStore for MemStore {
//!     fn put(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError> {
//!         self.0.insert(*hash, data.to_vec());
//!         Ok(())
//!     }
//!     fn get(&self, hash: &Hash) -> Result<Vec<u8>, VctrlError> {
//!         self.0.get(hash).cloned().ok_or(VctrlError::ObjectNotFound(*hash))
//!     }
//!     fn delete(&mut self, hash: &Hash) -> Result<(), VctrlError> {
//!         self.0.remove(hash);
//!         Ok(())
//!     }
//!     fn exists(&self, hash: &Hash) -> Result<bool, VctrlError> {
//!         Ok(self.0.contains_key(hash))
//!     }
//! }
//! ```
//!
//! ## Stability guarantees
//!
//! - The public API is covered by **semantic versioning**.
//! - The `#[non_exhaustive]` attribute on `EntryKind` and `VctrlError` allows
//!   adding new variants without a major version bump.
//! - Constants may only change value in a major release.
//!
//! ## Feature flags
//!
//! This crate intentionally has **no** feature flags. Every component is always
//! available. Specialised functionality (like a particular hash algorithm or
//! network transport) is provided by other crates in the workspace.
//!
//! ## License
//!
//! MIT – see the repository root for details.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc(html_root_url = "https://docs.rs/libvctrl_handler/1.0.0")]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod constants;
pub mod enums;
pub mod errors;
/// Convenience macros for working with errors.
pub mod macros;
pub mod traits;
pub mod types;

// Re-export fundamental items with explicit paths to avoid wildcard imports.
pub use constants::{
    HASH_LENGTH, MAX_BLOB_SIZE, MAX_MESSAGE_LENGTH, MAX_NAME_LENGTH, MAX_TREE_ENTRIES,
};
pub use enums::EntryKind;
pub use errors::VctrlError;
pub use traits::{Decoder, Encoder, Hasher, ObjectStore, RefStore, Signer, Transport, Verifier};
pub use types::{Blob, Commit, CommitMeta, Hash, Tag, Tree, TreeEntry, UserID};
