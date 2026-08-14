//! Fundamental contracts for building a version control system.
//!
//! # Purpose
//!
//! `libvctrl_handler` provides the core, pure-data types and behavior traits
//! required to construct a version control system (VCS). It intentionally
//! contains no implementations, only the abstract definitions of objects
//! (blobs, trees, commits, tags) and the interfaces for storing, hashing,
//! encoding, and transporting them.
//!
//! # Design Rationale
//!
//! The crate enforces a strict separation between data and behavior:
//!
//! - Data is represented by immutable structs in `types`.
//! - Behavior is defined by traits in `traits`.
//!
//! This decoupling allows downstream applications to mix and match backends
//! (for example, an in-memory store with a binary encoder and Ed25519
//! signing) without altering the core domain logic.
//!
//! ## Lint policy
//!
//! The crate uses a strict set of compiler and Clippy lints to ensure high
//! code quality. `clippy::nursery` is configured as a warning rather than a
//! deny because nursery lints are unstable and can introduce new warnings
//! with Rust toolchain updates. Critical nursery lints can still be denied
//! individually if needed.
//!
//! # Internal Mechanism
//!
//! The crate exports all public types, traits, and constants at the root level
//! for convenience. Consumers can use `libvctrl_handler::*;` to access the
//! entire contract surface. The re-exports mirror the internal module
//! structure:
//!
//! - Constants from `constants` are re-exported directly.
//! - Enums from `enums` are re-exported as `EntryKind`.
//! - Error types from `errors` are re-exported as `VctrlError`.
//! - Traits from `traits` are re-exported (for example `Hasher` and
//!   `ObjectStore`).
//! - Data types from `types` are re-exported (for example `Blob` and
//!   `Hash`).
//!
//! This flat namespace is ideal for a contract crate because it eliminates
//! excessive qualification in downstream code while still allowing selective
//! imports.
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
    clippy::cargo,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    unused_crate_dependencies,
    unused_qualifications
)]
// Nursery lints are unstable; we only warn so that toolchain updates do not
// suddenly break the build. See crate-level documentation for rationale.

/// System-wide constants and structural limits used across the version control
/// system.
///
/// # Purpose
///
/// This module centralizes all numeric constants (for example `HASH_LENGTH`
/// and `MAX_NAME_LENGTH`) so that they can be used consistently by every
/// other module and by downstream crates. Changing a constant here
/// automatically propagates to all dependent code.
///
/// # Why a separate module
///
/// Grouping constants in one module avoids circular dependencies and keeps
/// the root namespace clean. It also makes it easy to document each constant
/// with its own doctest.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::constants::HASH_LENGTH;
/// assert_eq!(HASH_LENGTH, 64);
/// ```
pub mod constants;

/// Logical object type enumerations, distinguishing between files and
/// directories.
///
/// # Purpose
///
/// The `EntryKind` enum is used throughout the system to differentiate
/// between a file (blob) and a directory (tree). It is deliberately kept
/// small to facilitate exhaustive matching.
///
/// # Design note
///
/// By using a C-like enum (no data attached), we ensure `EntryKind` is
/// `Copy`, lightweight, and easy to embed in other structures without
/// lifetime concerns.
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
/// # Purpose
///
/// The `errors` module exports the `crate::VctrlError` enum, which is the
/// single error type used by every trait method in this crate. This
/// unification simplifies error propagation and pattern matching for
/// consumers.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::errors::VctrlError;
///
/// let err = VctrlError::Other("fail".to_string());
/// assert_eq!(err.to_string(), "fail");
/// ```
pub mod errors;

/// Helper macros for ergonomic error construction.
///
/// # Purpose
///
/// The `vctrl_error_other!` macro provides a concise way to create
/// `crate::VctrlError::Other` variants with formatted messages, mimicking
/// the `format!` syntax.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::vctrl_error_other;
///
/// let err = vctrl_error_other!("code {}", 500);
/// assert_eq!(err.to_string(), "code 500");
/// ```
pub mod macros;

/// Core behavior contracts (traits) for storage, encoding, hashing, and
/// transport.
///
/// # Purpose
///
/// This module defines the interfaces that any concrete backend must
/// implement. By depending only on these traits, the core logic remains
/// completely decoupled from specific storage engines, hash algorithms, or
/// network transports.
///
/// # Design Rationale
///
/// Every trait follows the single responsibility principle:
///
/// - `crate::ObjectStore` handles object retrieval and storage.
/// - `crate::RefStore` manages named references (branches, tags).
/// - `crate::Hasher` computes cryptographic hashes.
/// - `crate::Encoder` and `crate::Decoder` serialize and deserialize objects.
/// - `crate::Signer` and `crate::Verifier` handle digital signatures.
/// - `crate::Transport` abstracts the network layer.
///
/// This separation allows a user to swap, for example, the hash algorithm
/// without touching any other component.
///
/// # Examples
///
/// Implementing a dummy `crate::Hasher`:
///
/// ```
/// use libvctrl_handler::{Hash, Hasher, VctrlError};
///
/// struct DummyHasher;
///
/// impl Hasher for DummyHasher {
///     fn hash(&self, _data: &[u8]) -> Result<Hash, VctrlError> {
///         Ok(Hash::from_bytes(&[0u8; 64]).unwrap())
///     }
/// }
///
/// let hasher = DummyHasher;
/// let hash = hasher.hash(b"hello").unwrap();
/// assert_eq!(hash.as_bytes().len(), 64);
/// ```
pub mod traits;

/// Core data structures representing version control objects.
///
/// # Purpose
///
/// The `types` module contains all the domain models: `crate::Blob`,
/// `crate::Tree`, `crate::Commit`, `crate::Tag`, and supporting types
/// like `crate::Hash` and `crate::UserID`. These structs are intentionally
/// immutable after construction to simplify reasoning about state and to
/// guarantee thread safety.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::types::Blob;
///
/// let blob = Blob::new(vec![1, 2, 3]);
/// assert_eq!(blob.size(), 3);
/// ```
pub mod types;

/// Re-exports of fundamental system constants like `HASH_LENGTH` and
/// maximum size limits.
///
/// # Purpose
///
/// These constants are used so frequently that they are re-exported at the
/// crate root. This saves the caller from having to write
/// `libvctrl_handler::constants::HASH_LENGTH` everywhere.
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

/// Re-export of the `EntryKind` enum.
///
/// `EntryKind` is the only public enum in the crate, and re-exporting it
/// at the root reinforces its role as a fundamental building block.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::EntryKind;
/// assert_eq!(EntryKind::Blob, EntryKind::Blob);
/// ```
pub use enums::EntryKind;

/// Re-export of the unified `VctrlError` type.
///
/// # Purpose
///
/// Every fallible operation in this crate returns `Result<_, VctrlError>`.
/// Making `VctrlError` available at the crate root streamlines error
/// handling for downstream code.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::VctrlError;
///
/// let err = VctrlError::Other("test".to_string());
/// assert!(err.to_string().contains("test"));
/// ```
pub use errors::VctrlError;

/// Re-exports of the core behavior traits.
///
/// This includes:
///
/// - `crate::ObjectStore`
/// - `crate::RefStore`
/// - `crate::Hasher`
/// - `crate::Encoder` and `crate::Decoder`
/// - `crate::Signer` and `crate::Verifier`
/// - `crate::Transport`
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Hash, Hasher, VctrlError};
///
/// struct MyHasher;
///
/// impl Hasher for MyHasher {
///     fn hash(&self, _data: &[u8]) -> Result<Hash, VctrlError> {
///         Ok(Hash::from_bytes(&[0u8; 64]).unwrap())
///     }
/// }
///
/// let hasher = MyHasher;
/// let hash = hasher.hash(b"data").unwrap();
/// assert_eq!(hash.as_bytes().len(), 64);
/// ```
pub use traits::core::{
    blame::{Blame, BlameEntry},
    config::ConfigStore,
    decoder::Decoder,
    diff::{Change, TreeDiffer},
    encoder::Encoder,
    hasher::Hasher,
    index::Index,
    object_store::ObjectStore,
    pack::{PackReader, PackWriter},
    ref_store::RefStore,
    reflog::ReflogStore,
    remote::Remote,
    revwalk::RevWalk,
    signer::Signer,
    transport::Transport,
    verifier::Verifier,
};
/// Re-exports of the core data structures.
///
/// All version-control objects (`crate::Blob`, `crate::Tree`,
/// `crate::Commit`, `crate::Tag`) and their supporting types
/// (`crate::Hash`, `crate::UserID`, `crate::CommitMeta`,
/// `crate::TreeEntry`) are available directly from the crate root.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::Blob;
///
/// let blob = Blob::new(vec![1, 2, 3]);
/// assert_eq!(blob.size(), 3);
/// ```
pub use types::{
    Blob, ChangeKind, Commit, CommitMeta, Conflict, FileDelta, Hash, MergeResult, ReflogEntry, Tag,
    Tree, TreeDelta, TreeEntry, UserID,
};
