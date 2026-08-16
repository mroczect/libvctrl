//! # libvctrl_handler
//!
//! A robust, pure-Rust implementation of Git internals, designed for
//! high-performance and enterprise-grade reliability.
//!
//! ## Architecture
//!
//! The crate is strictly separated into distinct domains of responsibility:
//!
//! - **[`constants`]**: Defines hard limits and magic numbers used across the crate to prevent
//!   unbounded memory allocation and ensure protocol compliance.
//! - **[`enums`]**: Provides exhaustive enumerations for Git-specific types, such as tree entry kinds.
//! - **[`errors`]**: Centralizes all error handling via the [`VctrlError`] enum, ensuring consistent
//!   error propagation and diagnostics.
//! - **[`macros`]**: Exposes declarative macros to reduce boilerplate for error construction.
//! - **[`traits`]**: Defines the core abstract behaviors (e.g., [`Encoder`], [`Decoder`], [`ObjectStore`]).
//!   This allows consumers to plug in their own backends (in-memory, filesystem, network).
//! - **[`types`]**: Contains strongly-typed representations of Git objects (e.g., [`Blob`], [`Tree`], [`Commit`]).
//! - **[`validation`]**: Provides pure functions to validate inputs like names, hashes, and references
//!   before they enter the system state.
//!
//! ## Safety and Idioms
//!
//! This crate enforces `#![forbid(unsafe_code)]` to guarantee memory safety without compromise.
//! It also aggressively denies clippy lints (all, pedantic, nursery) and enforces
//! `missing_docs` to ensure the public API is fully documented. The design relies on
//! Rust's zero-cost abstractions, utilizing `const fn` where possible to shift computations
//! to compile time.
//!
//! ## Examples
//!
//! *Note: The following examples assume this crate is named `libvctrl_handler`.*
//!
//! Creating a valid [`Hash`] and inspecting an [`EntryKind`]:
//!
//! ```
//! # use libvctrl_handler::{EntryKind, Hash};
//! // Hash requires exactly 64 bytes (SHA-512).
//! let raw_bytes = [0u8; 64];
//! let hash = Hash::from_bytes(&raw_bytes);
//! assert!(hash.is_ok());
//!
//! // Git object modes can be inspected via the EntryKind enum.
//! let blob_mode = EntryKind::Blob.mode();
//! assert_eq!(blob_mode, 0o100_644);
//! ```

/// Constants related to Git object formats and operational limits.
///
/// # Why this exists
/// Git has implicit and explicit limits (like maximum blob size or tree entries).
/// Centralizing these constants prevents magic numbers across the codebase and
/// ensures that limits are uniformly enforced at the type construction level.
pub mod constants;

/// Enums for Git object types.
///
/// # Why this exists
/// Using strongly-typed enums instead of raw integers (like `u32` mode bits)
/// allows the compiler to exhaustively match object kinds, preventing invalid states
/// and making the API self-documenting.
pub mod enums;

/// Error types used throughout the crate.
///
/// # Why this exists
/// Centralizes all error variants into a single [`VctrlError`] enum. This allows
/// consumers to handle errors uniformly using the `?` operator across different subsystems
/// without needing to box or wrap disparate error types manually.
pub mod errors;

/// Helper macros for the crate.
///
/// # Why this exists
/// Provides syntactic sugar for error creation, reducing boilerplate when wrapping
/// strings into [`VctrlError::Other`] and ensuring consistent error formatting.
pub mod macros;

/// Traits defining repository operations.
///
/// # Why this exists
/// By defining traits like [`ObjectStore`] or [`Encoder`], the crate decouples
/// the business logic from the underlying I/O backend. This enables mocking
/// for tests and allows for custom storage implementations (e.g., in-memory vs. disk).
pub mod traits;

/// Core data types for Git objects.
///
/// # Why this exists
/// Provides immutable, validated structures like [`Commit`] and [`Tree`].
/// Construction is fallible, ensuring that invalid objects cannot exist at runtime.
pub mod types;

/// Pure validation functions for Git inputs.
///
/// # Why this exists
/// Separating validation from data structures allows the same logic to be
/// applied to raw inputs before attempting object construction, failing fast
/// on malformed data and preventing invalid states from ever being created.
pub mod validation;

/// Re-exports of fundamental constants for easy access.
///
/// These limits are enforced during object construction to prevent memory exhaustion
/// and maintain Git protocol compliance.
pub use constants::{
    HASH_LENGTH, MAX_BLOB_SIZE, MAX_MESSAGE_LENGTH, MAX_NAME_LENGTH, MAX_PARENT_COUNT,
    MAX_TREE_ENTRIES,
};

/// Re-export of the [`EntryKind`] enum for classifying tree entries.
pub use enums::EntryKind;

/// Re-export of the primary error type [`VctrlError`].
pub use errors::VctrlError;

/// Re-exports of core operational traits for backend implementation.
///
/// Implement these traits to create a custom Git backend or to interact with
/// repository data generically.
pub use traits::core::{
    blame::{Blame, BlameEntry},
    config::ConfigStore,
    decoder::Decoder,
    diff::TreeDiffer,
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

/// Re-exports of strongly-typed Git object representations.
///
/// These types are the primary data carriers used in encoding, decoding, and manipulation.
pub use types::{
    Blob, ChangeKind, Commit, CommitMeta, Conflict, FileDelta, Hash, MergeResult, ReflogEntry, Tag,
    Tree, TreeDelta, TreeEntry, UserID,
};

/// Re-exports of validation utilities.
///
/// Use these functions to sanitize or verify inputs before passing them to constructors.
pub use validation::{
    validate_hash_bytes, validate_name, validate_ref_name, validate_tree_entry_name,
};
