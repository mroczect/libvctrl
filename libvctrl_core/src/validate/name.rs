//! Name validation utilities for `libvctrl_core`.
//!
//! # Purpose
//!
//! This module provides utility functions to validate the structural and
//! security constraints of names used in the version control system (e.g.,
//! branch names, tag names, tree entry names). The central function,
//! [`validate_name`], enforces a strict set of rules before a string is
//! accepted as a valid identifier.
//!
//! # Design Rationale
//!
//! - **Security defense**: The primary rationale is to prevent path traversal
//!   attacks. If a malicious actor supplies a name like `../../etc/passwd`,
//!   it could cause a naive filesystem backend to write or read outside the
//!   designated repository directory. By strictly forbidding slashes (`/`)
//!   and special directory names (`.` and `..`), this module enforces a safe
//!   namespace.
//! - **Resource exhaustion prevention**: Enforcing a maximum length
//!   ([`MAX_NAME_LENGTH`](libvctrl_handler::MAX_NAME_LENGTH)) prevents
//!   pathologically long names from causing excessive memory allocations or
//!   exceeding filesystem limits.
//! - **Centralized logic**: By centralizing these rules, all object builders
//!   and reference stores can delegate to this function, ensuring consistent
//!   validation across the entire system.
//!
//! # Relationship to `libvctrl_handler`
//!
//! The handler crate provides its own internal validation helpers for use in
//! its constructors. This module reimplements the same rules in a standalone
//! function so that `libvctrl_core` can validate names before passing them to
//! handler constructors. This avoids duplicating validation logic in multiple
//! backend implementations.
//!
//! # Security Considerations
//!
//! The validation rules are intentionally strict. A name must:
//!
//! 1. Be non-empty.
//! 2. Not exceed [`MAX_NAME_LENGTH`](libvctrl_handler::MAX_NAME_LENGTH)
//!    bytes.
//! 3. Not contain a forward slash (`/`), which is a path separator on
//!    Unix-like systems.
//! 4. Not be exactly `.` or `..`, which are special directory aliases.
//!
//! These rules are the minimum required to prevent directory traversal and
//! filesystem confusion. They do not enforce character-set restrictions
//! (e.g., forbidding control characters), which may be added later if needed.
//!
//! # Performance
//!
//! The function performs a constant number of checks and one linear scan for
//! the slash character. The overall time complexity is O(n), where n is the
//! length of the name. The checks are ordered from cheapest to most expensive
//! to fail fast on common invalid inputs.
//!
//! # Examples
//!
//! Validating a correct name:
//!
//! ```
//! use libvctrl_core::validate::name::validate_name;
//!
//! assert!(validate_name("feature_branch").is_ok());
//! assert!(validate_name("v1.0.0").is_ok());
//! ```
//!
//! Rejecting an empty name:
//!
//! ```
//! use libvctrl_core::validate::name::validate_name;
//! assert!(validate_name("").is_err());
//! ```
//!
//! Rejecting a name with a path separator:
//!
//! ```
//! use libvctrl_core::validate::name::validate_name;
//! assert!(validate_name("dir/file").is_err());
//! ```
//!
//! Rejecting directory aliases:
//!
//! ```
//! use libvctrl_core::validate::name::validate_name;
//! assert!(validate_name(".").is_err());
//! assert!(validate_name("..").is_err());
//! ```

use libvctrl_handler::{MAX_NAME_LENGTH, VctrlError};

/// Validates a name string against length and security rules.
///
/// # Purpose
///
/// This function acts as a gatekeeper for any string used as an identifier
/// or filename within the version control system. It returns `Ok(())` if the
/// name passes all checks, or an error describing the first failure.
///
/// # Design Rationale
///
/// The checks are ordered from cheapest to most expensive:
///
/// 1. Emptiness check (fast length check).
/// 2. Maximum length check (bounds resource usage).
/// 3. Slash containment check (prevents directory traversal).
/// 4. Exact match for `.` and `..` (prevents directory hijacking).
///
/// This ordering ensures that the most common invalid inputs fail quickly,
/// reducing the average cost of validation.
///
/// # Internal Mechanism
///
/// The function uses standard string slicing and searching methods. The
/// [`str::contains`] method is used for slash detection, which performs a
/// linear scan but is highly optimized in the standard library. The exact
/// equality checks for `.` and `..` are simple pointer comparisons after the
/// length and slash checks have already run.
///
/// # Errors
///
/// Returns
/// [`VctrlError::InvalidName`](libvctrl_handler::VctrlError::InvalidName)
/// if the name:
///
/// - Is empty.
/// - Exceeds
///   [`MAX_NAME_LENGTH`](libvctrl_handler::MAX_NAME_LENGTH).
/// - Contains a forward slash (`/`).
/// - Is exactly `.` or `..`.
///
/// The error message provides a descriptive reason for the failure.
///
/// # Panics
///
/// Panics if [`MAX_NAME_LENGTH`](libvctrl_handler::MAX_NAME_LENGTH) cannot
/// be converted to `usize`. This is a programmer error that indicates a
/// misconfigured constant on a platform where `usize` is too small to hold
/// the value. In practice this cannot happen on 32-bit or 64-bit systems.
///
/// # Examples
///
/// Validating a correct name:
///
/// ```
/// use libvctrl_core::validate::name::validate_name;
///
/// assert!(validate_name("feature_branch").is_ok());
/// assert!(validate_name("v1.0.0").is_ok());
/// ```
///
/// Rejecting an empty name:
///
/// ```
/// use libvctrl_core::validate::name::validate_name;
/// assert!(validate_name("").is_err());
/// ```
///
/// Rejecting a name with a path separator:
///
/// ```
/// use libvctrl_core::validate::name::validate_name;
/// assert!(validate_name("dir/file").is_err());
/// ```
///
/// Rejecting directory aliases:
///
/// ```
/// use libvctrl_core::validate::name::validate_name;
/// assert!(validate_name(".").is_err());
/// assert!(validate_name("..").is_err());
/// ```
pub fn validate_name(name: &str) -> Result<(), VctrlError> {
    if name.is_empty() {
        return Err(VctrlError::InvalidName("name is empty".into()));
    }
    if name.len() > usize::try_from(MAX_NAME_LENGTH).expect("MAX_NAME_LENGTH too large") {
        return Err(VctrlError::InvalidName(format!(
            "name exceeds maximum length {MAX_NAME_LENGTH}: '{name}'"
        )));
    }
    if name.contains('/') {
        return Err(VctrlError::InvalidName("name must not contain '/'".into()));
    }
    if name == "." || name == ".." {
        return Err(VctrlError::InvalidName(
            "name must not be '.' or '..'".into(),
        ));
    }
    Ok(())
}
