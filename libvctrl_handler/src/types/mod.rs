//! # Types
//!
//! Fundamental data types and name validation utilities for the version control system.
//!
//! This module houses all core types—blobs, trees, commits, tags, and hashes—in the `core`
//! submodule and re-exports them for ergonomic access. It also provides shared name validation
//! functions that enforce invariants across the system.
//!
//! ## Architecture
//!
//! - `core` module: contains the structs and enums that model the object model. These types
//!   implement parsing, serialization, and domain-specific behaviours.
//! - Re-export: `pub use core::*` lifts every type to `crate::types`, so consumers can write
//!   `use libvctrl_handler::types::Blob` instead of `libvctrl_handler::types::core::Blob`.
//! - Validation helpers: `validate_name` and `validate_tree_entry_name` are `pub(crate)`,
//!   intentionally scoped to the crate to prevent external misuse while remaining available
//!   to all internal modules.
//!
//! ## Crate name assumption
//!
//! For documentation doctests this module assumes the library crate is named `libvctrl_handler`.
//! Adjust import paths accordingly when integrating into a real project.
//!
//! # Examples
//!
//! Using a re-exported type:
//!
//! ```
//! use libvctrl_handler::types::Blob;
//! let blob = Blob::new(b"hello world".to_vec());
//! ```

/// Core object-model types.
///
/// This submodule defines the fundamental building blocks of the version control system:
/// [`Blob`], [`Tree`], [`Commit`], [`Tag`], [`Hash`], and supporting types like [`UserID`].
/// Each type is designed as a plain-old-data struct with immutable fields, mirroring the
/// content-addressable storage philosophy. They are intentionally `pub` so that external
/// consumers can construct and inspect them, while mutations remain the responsibility of
/// higher-level managers.
///
/// # How it fits
///
/// The `core` module is the source of truth. Other subsystems (`traits`, `handlers`, …)
/// depend on these types through re-exports from the parent `types` module, keeping
/// dependency graphs shallow and avoiding circular imports.
///
/// # Examples
///
/// Constructing a blob through the `core` path:
///
/// ```
/// use libvctrl_handler::types::core::Blob;
/// let blob = Blob::new(b"example data".to_vec());
/// ```
pub mod core;

use crate::constants::MAX_NAME_LENGTH;
use crate::errors::VctrlError;

/// Re-exports all types from `core` into the `types` namespace.
///
/// Without this re-export, consumers would need to write `use libvctrl_handler::types::core::Blob`.
/// By lifting them to `types`, we present a cleaner public API while keeping the
/// implementation modular.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::types::Blob;
/// let blob = Blob::new(b"lifted access".to_vec());
/// ```
pub use core::*;

/// Validates a general-purpose name (branch, tag, remote, etc.) against length constraints.
///
/// Names are required to be non-empty and not exceed [`MAX_NAME_LENGTH`] bytes.
/// This function is `pub(crate)` because name validation is an internal invariant;
/// external users should never be able to inject a name that bypasses these checks.
///
/// # How it works
///
/// 1. Checks emptiness → early `InvalidName` error.
/// 2. Checks length against the compile-time constant `MAX_NAME_LENGTH`.
///
/// # Errors
///
/// Returns [`VctrlError::InvalidName`] with a descriptive message when the name is empty
/// or too long.
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

/// Validates a name intended for a tree entry (file or directory name inside a tree object).
///
/// In addition to the checks performed by [`validate_name`], this function forbids:
/// - Slash characters (`/`), which would interfere with path parsing.
/// - The reserved names `.` and `..`, which have special meanings in Unix-like systems.
///
/// # Why it exists
///
/// Tree entries must be simple, flat names without directory separators. Enforcing this
/// at the validation layer prevents entire classes of path-traversal and tree-corruption
/// bugs before they reach storage.
///
/// # How it works
///
/// 1. Calls [`validate_name`] to enforce basic constraints.
/// 2. Checks for `/`, `.`, and `..` characters.
///
/// # Errors
///
/// Returns [`VctrlError::InvalidName`] if the name is empty, too long, or contains forbidden
/// characters/names.
pub(crate) fn validate_tree_entry_name(name: &str) -> Result<(), VctrlError> {
    validate_name(name)?;
    if name.contains('/') || name == "." || name == ".." {
        return Err(VctrlError::InvalidName(format!(
            "tree entry name contains forbidden path characters or names: '{name}'"
        )));
    }
    Ok(())
}
