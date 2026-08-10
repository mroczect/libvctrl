//! Validation utilities for `libvctrl_core`.
//!
//! # Purpose
//! This module provides centralized, reusable validation functions for
//! core version control entities, specifically cryptographic hashes and
//! naming strings.
//!
//! # Design rationale
//! - **Separation of Concerns**: Validation logic is deliberately separated
//!   from the pure data structs in [`libvctrl_handler`]. This keeps the data
//!   types lightweight and allows callers to perform pre-checks on untrusted
//!   inputs before attempting to construct objects.
//! - **Security and Integrity**: The functions here enforce critical security
//!   constraints, such as preventing path traversal attacks via malicious
//!   names, and ensuring structural integrity, like verifying the exact byte
//!   length of a hash.
//!
//! # Internal mechanism
//! The module is split into submodules targeting specific entity types.
//! Each function performs a series of checks and returns a
//! [`VctrlError`](libvctrl_handler::VctrlError) on the first failure,
//! failing fast to conserve resources.

/// Module containing hash validation utilities.
///
/// # Purpose
/// Ensures that byte slices intended to represent hashes meet the strict
/// length requirements before being converted to the [`Hash`](libvctrl_handler::Hash)
/// type.
///
/// # Examples
///
/// ```
/// use libvctrl_core::validate::hash::validate_hash_bytes;
/// use libvctrl_handler::HASH_LENGTH;
///
/// let valid_bytes = [0u8; HASH_LENGTH];
/// assert!(validate_hash_bytes(&valid_bytes).is_ok());
/// ```
pub mod hash;

/// Module containing name validation utilities.
///
/// # Purpose
/// Ensures that strings used as identifiers (e.g., branches, tags, filenames)
/// are non-empty, within length limits, and free of path traversal characters.
///
/// # Examples
///
/// ```
/// use libvctrl_core::validate::name::validate_name;
///
/// assert!(validate_name("valid_name").is_ok());
/// assert!(validate_name("../invalid").is_err());
/// ```
pub mod name;
