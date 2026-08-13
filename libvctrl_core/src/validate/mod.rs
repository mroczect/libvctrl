//! Validation utilities for `libvctrl_core`.
//!
//! # Purpose
//!
//! This module provides centralized, reusable validation functions for core
//! version control entities, specifically cryptographic hashes and naming
//! strings. These functions act as gatekeepers that inspect raw, untrusted
//! input before it is converted into strongly typed objects.
//!
//! # Design Rationale
//!
//! - **Separation of concerns**: Validation logic is deliberately separated
//!   from the pure data structs in [`libvctrl_handler`]. This keeps the data
//!   types lightweight and allows callers to perform pre-checks on untrusted
//!   inputs before attempting to construct objects.
//! - **Security and integrity**: The functions here enforce critical security
//!   constraints, such as preventing path traversal attacks via malicious
//!   names, and ensuring structural integrity, like verifying the exact byte
//!   length of a hash.
//! - **Fail fast**: Each function performs a series of checks and returns a
//!   [`VctrlError`](libvctrl_handler::VctrlError) on the first failure. This
//!   minimizes wasted computation and provides immediate, descriptive errors.
//! - **Centralized policy**: By centralizing validation rules, every backend
//!   and builder in the crate can delegate to the same functions, ensuring
//!   consistent behavior across all entry points.
//!
//! # Internal Mechanism
//!
//! The module is split into submodules targeting specific entity types.
//! Each submodule exposes at least one validator function:
//!
//! - [`hash`](self::hash) contains
//!   [`validate_hash_bytes`](self::hash::validate_hash_bytes), which checks
//!   the exact length of a raw hash byte slice.
//! - [`name`](self::name) contains
//!   [`validate_name`](self::name::validate_name), which checks structural
//!   and security invariants of identifier strings.
//!
//! Both validators return `Result<(), VctrlError>` and never panic on
//! arbitrary input.
//!
//! # Relationship to `libvctrl_handler`
//!
//! `libvctrl_handler` already performs validation inside its constructors.
//! The validators in this module do not replace those checks; rather, they
//! duplicate the same rules in standalone functions so that higher-level
//! code can validate *before* invoking a constructor. This is useful when a
//! backend wants custom logging, custom error wrapping, or needs to reject
//! bad input before entering a broader transaction.
//!
//! # When to Use
//!
//! - Before calling [`Hash::from_bytes`](libvctrl_handler::Hash::from_bytes)
//!   on data read from disk or network.
//! - Before passing a user-provided string into a constructor such as
//!   [`Tag::new`](libvctrl_handler::Tag::new) or
//!   [`TreeEntry::new`](libvctrl_handler::TreeEntry::new).
//! - Inside custom storage or codec backends that want explicit, contextual
//!   validation errors.
//!
//! # Examples
//!
//! Validating a hash and a name:
//!
//! ```
//! use libvctrl_core::validate::hash::validate_hash_bytes;
//! use libvctrl_core::validate::name::validate_name;
//! use libvctrl_handler::HASH_LENGTH;
//!
//! let hash_bytes = [0u8; HASH_LENGTH];
//! assert!(validate_hash_bytes(&hash_bytes).is_ok());
//!
//! assert!(validate_name("feature-branch").is_ok());
//! assert!(validate_name("../bad").is_err());
//! ```

/// Module containing hash validation utilities.
///
/// # Purpose
///
/// Ensures that byte slices intended to represent hashes meet the strict
/// length requirements before being converted to the
/// [`Hash`](libvctrl_handler::Hash) type.
///
/// # Design Rationale
///
/// Hash length validation is isolated here so that all callers can perform
/// a cheap pre-check without depending on the `Hash` type's internal
/// constructor. This is especially useful in codecs and storage backends
/// that read raw bytes from untrusted sources.
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
///
/// Ensures that strings used as identifiers (e.g., branches, tags,
/// filenames) are non-empty, within length limits, and free of path
/// traversal characters.
///
/// # Design Rationale
///
/// Names are the most common input from users and therefore the most likely
/// attack vector. This module centralizes security-critical name checks to
/// prevent path traversal and resource exhaustion consistently across all
/// backends.
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
