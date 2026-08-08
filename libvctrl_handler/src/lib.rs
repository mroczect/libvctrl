#![doc = include_str!("../README.md")]

//! # `libvctrl_handler` – The Unshakeable Contract
//!
//! This crate **only** defines the fundamental traits, types, errors, and constants
//! for building a version control system. **No implementations are allowed here.**
//!
//! It is the single source of truth for the entire `libvctrl` ecosystem.
//! Every other component must depend on this crate and must never redefine
//! these fundamental contracts.
//!
//! ## Philosophy
//! - **Mechanism, not policy** – no assumptions about branches, workflows, or defaults.
//! - **Unbounded flexibility, high discipline** – everything is generic and replaceable,
//!   but every input is strictly validated.
//! - **This crate is the constitution** – all fundamental traits, types, and errors
//!   live exclusively here.
//!
//! ## Usage
//! ```rust
//! use libvctrl_handler::*;
//!
//! let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
//! let entry = TreeEntry::new("file.txt".into(), EntryKind::Blob, hash).unwrap();
//!
//! let err = VctrlError::Other(format!("something went wrong: {}", 42));
//! ```

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
pub use constants::{HASH_LENGTH, MAX_NAME_LENGTH};
pub use enums::EntryKind;
pub use errors::VctrlError;
pub use traits::{Decoder, Encoder, Hasher, ObjectStore, RefStore, Signer, Transport, Verifier};
pub use types::{Blob, Commit, Hash, Tag, Tree, TreeEntry, UserID};
