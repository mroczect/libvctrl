//! # Types
//!
//! Fundamental data types and name validation utilities for the version
//! control system.
//!
//! # Purpose
//!
//! This module is the canonical home for all domain model types used by
//! `libvctrl_handler`. It contains the core object types (blobs, trees,
//! commits, tags, hashes) and supporting identity types (user IDs), as well
//! as shared validation functions that enforce system-wide invariants.
//!
//! # Architecture
//!
//! The module is organised into two layers:
//!
//! - [`core`](self::core): the internal submodule where each type is defined
//!   in its own file. This keeps compilation units small and dependencies
//!   explicit.
//! - Re-exports: `pub use core::*` lifts every type to the `types`
//!   namespace, so consumers can write `use libvctrl_handler::types::Blob`
//!   instead of the longer `libvctrl_handler::types::core::blob::Blob`.
//!
//! ## Why a `core` submodule?
//!
//! Grouping type definitions in `core` provides a clean separation between
//! the internal module layout and the public API. The public path remains
//! stable even if the internal file organisation changes. This mirrors the
//! pattern used by the [`traits`](crate::traits) module.
//!
//! ## Validation helpers
//!
//! Two validation functions are provided:
//!
//! - [`validate_name`]: checks general-purpose names (branches, tags, etc.)
//!   for non-emptiness and maximum length.
//! - [`validate_tree_entry_name`]: extends [`validate_name`] with extra
//!   checks for tree entry names to prevent path traversal and reserved
//!   names.
//!
//! Both are `pub(crate)` because name validation is an internal invariant.
//! External users should never be able to inject a name that bypasses these
//! checks; constructors like [`Tag::new`](crate::Tag::new) and
//! [`TreeEntry::new`](crate::TreeEntry::new) call them automatically.
//!
//! # Design Rationale
//!
//! - **Immutability by default**: All types in `core` have private fields
//!   and public constructors. Once created, they cannot be mutated. This
//!   reflects the content-addressable storage philosophy: an object's hash
//!   is derived from its bytes, so mutating the object would change its
//!   identity.
//! - **Validation at construction**: Constructors return `Result` to force
//!   callers to handle invalid input immediately. This prevents malformed
//!   objects from entering the system.
//! - **Re-export ergonomics**: Lifting all types to the parent module
//!   simplifies imports without sacrificing internal organisation.
//!
//! # Crate name assumption
//!
//! For documentation doctests this module assumes the library crate is named
//! `libvctrl_handler`. Adjust import paths accordingly when integrating into
//! a real project.
//!
//! # Examples
//!
//! Using a re-exported type:
//!
//! ```
//! use libvctrl_handler::types::Blob;
//!
//! let blob = Blob::new(b"hello world".to_vec());
//! assert_eq!(blob.size(), 11);
//! ```

/// Core object-model types.
///
/// # Purpose
///
/// This submodule defines the fundamental building blocks of the version
/// control system: [`Blob`], [`Tree`], [`Commit`], [`Tag`], [`Hash`], and
/// supporting types like [`UserID`], [`CommitMeta`], and [`TreeEntry`].
/// Each type is designed as a plain-old-data struct with immutable fields,
/// mirroring the content-addressable storage philosophy.
///
/// # Design Rationale
///
/// Each type is placed in its own file for maintainability and to reduce
/// merge conflicts. The types are intentionally public so that external
/// consumers can construct and inspect them, while mutations remain the
/// responsibility of higher-level managers. The fields remain private to
/// preserve invariants established during construction.
///
/// # How It Fits
///
/// The `core` module is the source of truth. Other subsystems (`traits`,
/// `handlers`, encoders, decoders) depend on these types through re-exports
/// from the parent `types` module, keeping dependency graphs shallow and
/// avoiding circular imports.
///
/// # Examples
///
/// Constructing a blob through the `core` path:
///
/// ```
/// use libvctrl_handler::types::core::Blob;
///
/// let blob = Blob::new(b"example data".to_vec());
/// assert_eq!(blob.data(), b"example data");
/// ```
pub mod core;

use crate::constants::MAX_NAME_LENGTH;
use crate::errors::VctrlError;

/// Re-exports all types from `core` into the `types` namespace.
///
/// # Purpose
///
/// Without this re-export, consumers would need to write
/// `use libvctrl_handler::types::core::Blob;`. By lifting them to `types`,
/// we present a cleaner public API while keeping the implementation modular.
///
/// # Design Rationale
///
/// This re-export is an ergonomic convenience. It does not duplicate the
/// types; it merely exposes them at a shallower path. This is a common Rust
/// pattern for modules that contain many public items.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::types::Blob;
///
/// let blob = Blob::new(b"lifted access".to_vec());
/// assert_eq!(blob.size(), 13);
/// ```
pub use core::*;

/// Validates a general-purpose name (branch, tag, remote, etc.) against
/// length constraints.
///
/// Names are required to be non-empty and not exceed
/// [`MAX_NAME_LENGTH`](crate::constants::MAX_NAME_LENGTH) bytes.
///
/// # Purpose
///
/// This function is `pub(crate)` because name validation is an internal
/// invariant; external users should never be able to inject a name that
/// bypasses these checks. Public constructors call this function before
/// accepting a name.
///
/// # How It Works
///
/// 1. Checks emptiness, returning `VctrlError::InvalidName` if empty.
/// 2. Checks length against the compile-time constant `MAX_NAME_LENGTH`.
/// 3. Returns `Ok(())` if all checks pass.
///
/// # Errors
///
/// Returns [`VctrlError::InvalidName`] with a descriptive message when the
/// name is empty or too long.
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn validate_name(name: &str) -> Result<(), VctrlError> {
    if name.is_empty() {
        return Err(VctrlError::InvalidName("name is empty".into()));
    }
    if name.len() > MAX_NAME_LENGTH as usize {
        return Err(VctrlError::InvalidName(format!(
            "name exceeds maximum length {MAX_NAME_LENGTH}: '{name}'"
        )));
    }
    Ok(())
}

/// Validates a name intended for a tree entry (file or directory name inside
/// a tree object).
///
/// # Purpose
///
/// In addition to the checks performed by [`validate_name`], this function
/// forbids:
///
/// - Slash characters (`/`), which would interfere with path parsing.
/// - The reserved names `.` and `..`, which have special meanings in
///   Unix-like systems.
///
/// # Why It Exists
///
/// Tree entries must be simple, flat names without directory separators.
/// Enforcing this at the validation layer prevents entire classes of
/// path-traversal and tree-corruption bugs before they reach storage.
///
/// # How It Works
///
/// 1. Calls [`validate_name`] to enforce basic constraints.
/// 2. Checks for `/`, `.`, and `..`.
/// 3. Returns `Ok(())` if all checks pass.
///
/// # Errors
///
/// Returns [`VctrlError::InvalidName`] if the name is empty, too long, or
/// contains forbidden characters or names.
pub(crate) fn validate_tree_entry_name(name: &str) -> Result<(), VctrlError> {
    validate_name(name)?;
    if name.contains('/') || name == "." || name == ".." {
        return Err(VctrlError::InvalidName(format!(
            "tree entry name contains forbidden path characters or names: '{name}'"
        )));
    }
    Ok(())
}
