//! Name validation utilities for `libvctrl_core`.
//!
//! # Purpose
//! This module provides utility functions to validate the structural and
//! security constraints of names used in the version control system (e.g.,
//! branch names, tag names, tree entry names).
//!
//! # Design rationale
//! - **Security Defense**: The primary rationale is to prevent path traversal
//!   attacks. If a malicious actor supplies a name like `../../etc/passwd`, it
//!   could cause a naive filesystem backend to write or read outside the
//!   designated repository directory. By strictly forbidding slashes (`/`) and
//!   special directory names (`.` and `..`), this module enforces a safe
//!   namespace.
//! - **Resource Exhaustion Prevention**: Enforcing a maximum length
//!   ([`MAX_NAME_LENGTH`](libvctrl_handler::MAX_NAME_LENGTH)) prevents
//!   pathologically long names from causing excessive memory allocations or
//!   exceeding filesystem limits.
//! - **Centralized Logic**: By centralizing these rules, all object builders
//!   and reference stores can delegate to this function, ensuring consistent
//!   validation across the entire system.

use libvctrl_handler::{MAX_NAME_LENGTH, VctrlError};

/// Validates a name string against length and security rules.
///
/// # Purpose
/// This function acts as a gatekeeper for any string used as an identifier
/// or filename within the version control system.
///
/// # Design rationale
/// The checks are ordered from cheapest to most expensive:
/// 1. Emptiness check (fast length check).
/// 2. Maximum length check (bounds resource usage).
/// 3. Slash containment check (prevents directory traversal).
/// 4. Exact match for `.` and `..` (prevents directory hijacking).
///
/// # Internal mechanism
/// It uses standard string slicing and searching methods. The
/// [`str::contains`] method is used for slash detection, which performs a
/// linear scan but is highly optimized in the standard library.
///
/// # Errors
/// Returns [`VctrlError::InvalidName`](libvctrl_handler::VctrlError::InvalidName)
/// if the name is empty, exceeds [`MAX_NAME_LENGTH`](libvctrl_handler::MAX_NAME_LENGTH),
/// contains a forward slash (`/`), or is exactly `.` or `..`.
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
    if name.len() > MAX_NAME_LENGTH {
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
