//! Fundamental contracts for building a version control system.
//!
//! # Purpose
//! `libvctrl_handler` provides the core, pure-data types and behavior traits
//! required to construct a version control system (VCS). It intentionally
//! contains *no implementations*—only the abstract definitions of objects
//! (blobs, trees, commits, tags) and the interfaces for storing, hashing,
//! encoding, and transporting them.
//!
//! # Design rationale
//! The crate enforces a strict separation between data and behavior:
//! - **Data** is represented by immutable structs in [`types`].
//! - **Behavior** is defined by traits in [`traits`].
//!
//! This decoupling allows downstream applications to mix and match backends
//! (e.g., an in-memory store with a binary encoder and Ed25519 signing) without
//! altering the core domain logic. The crate is built with strict Clippy lints
//! (`pedantic`, `nursery`) and forbids `unsafe` code to guarantee memory safety
//! and high code quality.
//!
//! # Internal mechanism
//! The crate exports all public types, traits, and constants at the root level
//! for convenience. Consumers can simply `use libvctrl_handler::*;` to access
//! the entire contract surface.
//!
//! # Examples
//!
//! Constructing a basic object and hash:
//!
//! ```
//! use libvctrl_handler::{Blob, Hash};
//!
//! let blob = Blob::new(b"content".to_vec());
//! let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
//! assert_eq!(blob.size(), 7);
//! assert_eq!(hash.as_bytes().len(), 64);
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

/// System-wide constants and structural limits used across the version control system.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::constants::HASH_LENGTH;
/// assert_eq!(HASH_LENGTH, 64);
/// ```
pub mod constants;

/// Logical object type enumerations, distinguishing between files and directories.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::enums::EntryKind;
/// assert_ne!(EntryKind::Blob, EntryKind::Tree);
/// ```
pub mod enums;

/// Unified error handling for all fallible operations within the crate.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::errors::VctrlError;
/// let err = VctrlError::Other("fail".to_string());
/// assert_eq!(err.to_string(), "fail");
/// ```
pub mod errors;

/// Helper macros for ergonomic error construction.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::VctrlError;
/// use libvctrl_handler::vctrl_error_other;
///
/// let err: VctrlError = vctrl_error_other!("code {}", 500);
/// assert_eq!(err.to_string(), "code 500");
/// ```
pub mod macros;

/// Core behavior contracts (traits) for storage, encoding, hashing, and transport.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::traits::Hasher;
/// use libvctrl_handler::Hash;
///
/// struct DummyHasher;
/// impl Hasher for DummyHasher {
///     fn hash(&self, _data: &[u8]) -> Hash {
///         Hash::from_bytes(&[0u8; 64]).unwrap()
///     }
/// }
/// ```
pub mod traits;

/// Core data structures representing version control objects.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::types::Blob;
/// let blob = Blob::new(vec![1, 2, 3]);
/// assert_eq!(blob.size(), 3);
/// ```
pub mod types;

/// Re-exports of fundamental system constants like [`HASH_LENGTH`](crate::constants::HASH_LENGTH) and maximum size limits.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::HASH_LENGTH;
/// assert_eq!(HASH_LENGTH, 64);
/// ```
pub use constants::{
    HASH_LENGTH, MAX_BLOB_SIZE, MAX_MESSAGE_LENGTH, MAX_NAME_LENGTH, MAX_TREE_ENTRIES,
};

/// Re-export of the [`EntryKind`](crate::enums::EntryKind) enum.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::EntryKind;
/// assert_eq!(EntryKind::Blob, EntryKind::Blob);
/// ```
pub use enums::EntryKind;

/// Re-export of the unified [`VctrlError`](crate::errors::VctrlError) type.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::VctrlError;
/// let err = VctrlError::Other("test".to_string());
/// assert!(err.to_string().contains("test"));
/// ```
pub use errors::VctrlError;

/// Re-exports of the core behavior traits (e.g., [`ObjectStore`](crate::traits::ObjectStore), [`Hasher`](crate::traits::Hasher)).
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Hasher, Hash};
///
/// struct MyHasher;
/// impl Hasher for MyHasher {
///     fn hash(&self, _data: &[u8]) -> Hash {
///         Hash::from_bytes(&[0u8; 64]).unwrap()
///     }
/// }
/// ```
pub use traits::{Decoder, Encoder, Hasher, ObjectStore, RefStore, Signer, Transport, Verifier};

/// Re-exports of the core data structures (e.g., [`Blob`](crate::types::Blob), [`Commit`](crate::types::Commit)).
///
/// # Examples
///
/// ```
/// use libvctrl_handler::Blob;
/// let blob = Blob::new(vec![1, 2, 3]);
/// assert_eq!(blob.size(), 3);
/// ```
pub use types::{Blob, Commit, CommitMeta, Hash, Tag, Tree, TreeEntry, UserID};
