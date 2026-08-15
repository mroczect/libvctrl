//! Fundamental contracts for building a version control system.
//!
//! # Purpose
//!
//! `libvctrl_handler` provides the core, pure-data types and behavior traits
//! required to construct a version control system. It intentionally contains
//! no implementations, only the abstract definitions of objects such as
//! blobs, trees, commits, and tags, and the interfaces for storing, hashing,
//! encoding, signing, and transporting them.
//!
//! # Why This Crate Exists
//!
//! A version control system consists of many moving parts: content-addressed
//! storage, named references, cryptographic hashing, serialization formats,
//! signature verification, and network transports. If those parts are coupled
//! together, replacing one component forces changes across the entire system.
//!
//! By defining only contracts, this crate allows downstream crates to:
//!
//! - Choose any storage backend, including memory, files, databases, or
//!   object stores that do not exist yet.
//! - Choose any hashing algorithm, as long as it produces a 64-byte digest.
//! - Choose any serialization format, as long as the resulting bytes can be
//!   decoded into the same domain object.
//! - Choose any signing and verification mechanism.
//! - Replace the transport layer without touching higher-level logic.
//!
//! # Design Rationale
//!
//! The crate enforces a strict separation between data and behavior:
//!
//! - Data is represented by immutable structs in [`types`].
//! - Behavior is defined by traits in [`traits`].
//! - System-wide limits and identifiers are centralized in [`constants`].
//! - All failures are represented by a single [`VctrlError`] type.
//!
//! This decoupling allows downstream applications to mix and match backends
//! without altering the core domain logic.
//!
//! ## Immutability and Content Addressing
//!
//! Every domain object is immutable after construction. A [`Blob`] cannot be
//! mutated after it is created; a [`Commit`] cannot have its parents changed.
//! This property is critical for content-addressed storage because the hash
//! of an object must remain stable for the lifetime of that object.
//!
//! ## No Hidden State
//!
//! There is no global mutable state, no environment variables, no implicit
//! threading, and no filesystem access in this crate. Every dependency is
//! injected through trait parameters. This keeps the contracts pure and
//! testable.
//!
//! ## Strict Validation
//!
//! All constructors validate their inputs. An invalid hash length, an empty
//! name, an excessively long message, or an unsorted tree entry list is
//! rejected immediately with [`VctrlError`]. This ensures that invalid objects
//! never enter downstream storage.
//!
//! # Architectural Overview
//!
//! The crate is split into six public modules:
//!
//! - [`constants`]: Numeric limits, hash length, and raw Unix mode bits.
//! - [`enums`]: Logical discriminator for tree entries, [`EntryKind`].
//! - [`errors`]: The unified [`VctrlError`] type.
//! - [`macros`]: Ergonomics helpers for constructing errors.
//! - [`traits`]: Storage, hashing, encoding, signing, and transport contracts.
//! - [`types`]: Immutable domain objects such as [`Blob`], [`Tree`],
//!   [`Commit`], [`Tag`], [`Hash`], and [`UserID`].
//!
//! The flat namespace at the crate root re-exports these items so that
//! downstream code can use concise imports.
//!
//! # Lint Policy
//!
//! The crate uses a strict set of compiler and Clippy lints to ensure high
//! code quality. `clippy::nursery` is configured as a warning rather than a
//! deny because nursery lints are unstable and can introduce new warnings
//! with Rust toolchain updates. Critical nursery lints can still be denied
//! individually if needed.
//!
//! The crate also forbids `unsafe` code entirely. This decision eliminates an
//! entire class of memory-safety bugs and makes the contracts easier to audit.
//!
//! # Internal Mechanism
//!
//! The crate exports all public types, traits, and constants at the root level
//! for convenience. Consumers can use `use libvctrl_handler::*;` to access the
//! entire contract surface. The re-exports mirror the internal module
//! structure:
//!
//! - Constants from [`constants`] are re-exported directly.
//! - Enums from [`enums`] are re-exported as [`EntryKind`].
//! - Error types from [`errors`] are re-exported as [`VctrlError`].
//! - Traits from [`traits`] are re-exported, for example [`Hasher`] and
//!   [`ObjectStore`].
//! - Data types from [`types`] are re-exported, for example [`Blob`] and
//!   [`Hash`].
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
#![warn(clippy::nursery)]

/// System-wide constants and structural limits used across the version control
/// system.
///
/// # Purpose
///
/// This module centralizes all numeric constants, such as [`HASH_LENGTH`] and
/// [`MAX_NAME_LENGTH`], so that they can be used consistently by every other
/// module and by downstream crates. Changing a constant here automatically
/// propagates to all dependent code.
///
/// # Why a Separate Module
///
/// Grouping constants in one module avoids circular dependencies and keeps the
/// root namespace clean. It also makes it easy to document each constant with
/// its own doctest. Constants are compile-time values, so placing them in a
/// dedicated module allows the optimizer to inline them without cost.
///
/// # How the Constants Are Used
///
/// The crate root re-exports the most commonly used constants directly. For
/// example, [`HASH_LENGTH`] is used by [`Hash`] to validate byte lengths, and
/// [`MAX_NAME_LENGTH`] is used by [`UserID`] and [`TreeEntry`] to reject
/// oversized names.
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
/// The [`EntryKind`] enum is used throughout the system to differentiate
/// between a regular file, an executable file, a symlink, a directory, and a
/// submodule reference. It is deliberately kept small to facilitate exhaustive
/// matching.
///
/// # Design Note
///
/// By using a C-like enum with no attached data, [`EntryKind`] is [`Copy`],
/// lightweight, and easy to embed in other structures without lifetime
/// concerns. The enum is also marked `#[non_exhaustive]`, which allows future
/// variants to be added without breaking downstream pattern matches.
///
/// # How It Integrates
///
/// [`TreeEntry`] stores an [`EntryKind`] together with a name and a [`Hash`].
/// Serialization backends can map [`EntryKind`] to their own on-disk mode bits
/// without coupling the logical kind to a particular filesystem convention.
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
/// The `errors` module exports [`VctrlError`], which is the single error type
/// used by every trait method in this crate. This unification simplifies error
/// propagation and pattern matching for consumers.
///
/// # Why One Error Type
///
/// A version control system can fail for many reasons: invalid data, missing
/// objects, I/O failures, serialization errors, and more. If each module had
/// its own error type, composing them would require error-conversion boilerplate.
/// A single non-exhaustive enum keeps the API small and predictable while still
/// allowing backward-compatible additions.
///
/// # How Errors Propagate
///
/// Every fallible function returns `Result<T, VctrlError>`. There are no panics
/// in the library. The [`VctrlError`] type implements `Display`, `Error`, and
/// `PartialEq`, so callers can display, inspect, and compare failures.
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
/// The [`vctrl_error_other!`] macro provides a concise way to create
/// [`VctrlError::Other`] variants with formatted messages, mimicking the
/// `format!` syntax. This reduces the boilerplate of writing
/// `VctrlError::Other(format!(...))` repeatedly.
///
/// # Why a Macro
///
/// A macro can accept format-string syntax and capture variable arguments
/// without forcing the caller to write an explicit `format!`. It is expanded
/// at compile time and has no runtime overhead.
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
/// This module defines the interfaces that any concrete backend must implement.
/// By depending only on these traits, the core logic remains completely
/// decoupled from specific storage engines, hash algorithms, or network
/// transports.
///
/// # Design Rationale
///
/// Every trait follows the single responsibility principle:
///
/// - [`ObjectStore`] handles object retrieval and storage.
/// - [`RefStore`] manages named references such as branches and tags.
/// - [`Hasher`] computes cryptographic hashes.
/// - [`Encoder`] and [`Decoder`] serialize and deserialize objects.
/// - [`Signer`] and [`Verifier`] handle digital signatures.
/// - [`Transport`] abstracts the network layer.
///
/// This separation allows a user to swap, for example, the hash algorithm
/// without touching any other component.
///
/// # How Traits Are Used
///
/// Downstream crates implement these traits for concrete backends. Higher-level
/// logic accepts the traits as generic parameters or trait objects, never as
/// concrete types. This ensures the same business logic can run on an
/// in-memory database during tests and on a distributed object store in
/// production.
///
/// # Examples
///
/// Implementing a dummy [`Hasher`]:
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
/// The `types` module contains all the domain models: [`Blob`], [`Tree`],
/// [`Commit`], [`Tag`], and supporting types like [`Hash`] and [`UserID`].
/// These structs are intentionally immutable after construction to simplify
/// reasoning about state and to guarantee thread safety.
///
/// # Why Immutable Structs
///
/// Version control objects are content-addressed. A hash identifies the exact
/// content of an object. If the content could be mutated after the hash was
/// computed, the hash would become invalid and the system would lose integrity.
/// Immutable structs make that class of bugs impossible at compile time.
///
/// # How Validation Works
///
/// Each constructor validates its input and returns a [`Result`]. For example,
/// [`TreeEntry::new`] checks that the name is non-empty and does not exceed
/// [`MAX_NAME_LENGTH`]. If validation fails, the constructor returns
/// [`VctrlError`] instead of allowing an invalid object to exist.
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

/// Re-exports of fundamental system constants like [`HASH_LENGTH`] and maximum
/// size limits.
///
/// # Purpose
///
/// These constants are used so frequently that they are re-exported at the
/// crate root. This saves the caller from having to write
/// `libvctrl_handler::constants::HASH_LENGTH` everywhere.
///
/// # How to Use
///
/// Import the constants directly:
///
/// ```
/// use libvctrl_handler::HASH_LENGTH;
/// assert_eq!(HASH_LENGTH, 64);
/// ```
pub use constants::{
    HASH_LENGTH, MAX_BLOB_SIZE, MAX_MESSAGE_LENGTH, MAX_NAME_LENGTH, MAX_TREE_ENTRIES,
};

/// Re-export of the [`EntryKind`] enum.
///
/// [`EntryKind`] is the only public enum in the crate, and re-exporting it at
/// the root reinforces its role as a fundamental building block.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::EntryKind;
/// assert_eq!(EntryKind::Blob, EntryKind::Blob);
/// ```
pub use enums::EntryKind;

/// Re-export of the unified [`VctrlError`] type.
///
/// # Purpose
///
/// Every fallible operation in this crate returns `Result<_, VctrlError>`.
/// Making [`VctrlError`] available at the crate root streamlines error handling
/// for downstream code.
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
/// - [`Blame`] and [`BlameEntry`] for line-level attribution.
/// - [`ConfigStore`] for configuration persistence.
/// - [`Decoder`] for object deserialization.
/// - [`Change`] and [`TreeDiffer`] for tree diffing.
/// - [`Encoder`] for object serialization.
/// - [`Hasher`] for cryptographic hashing.
/// - [`Index`] for staging-area abstraction.
/// - [`ObjectStore`] for content-addressed storage.
/// - [`PackReader`] and [`PackWriter`] for packfile handling.
/// - [`RefStore`] for named reference management.
/// - [`ReflogStore`] for reference logs.
/// - [`Remote`] for remote repository interactions.
/// - [`RevWalk`] for walking commit graphs.
/// - [`Signer`] and [`Verifier`] for digital signatures.
/// - [`Transport`] for object transport.
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
/// All version-control objects, such as [`Blob`], [`Tree`], [`Commit`], and
/// [`Tag`], and their supporting types such as [`Hash`], [`UserID`],
/// [`CommitMeta`], and [`TreeEntry`], are available directly from the crate
/// root.
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
