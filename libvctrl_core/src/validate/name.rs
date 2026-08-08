//! Name validation utilities.
//!
//! This module provides [`validate_name`], the single point of truth for
//! checking whether a string can be used as a name in the `libvctrl`
//! ecosystem.
//!
//! # What makes a valid name?
//!
//! A valid name must:
//! - Not be empty.
//! - Not exceed [`MAX_NAME_LENGTH`] bytes (255 by default).
//! - Not contain the path separator `/`.
//! - Not be exactly `.` or `..` (to prevent directory traversal).
//!
//! These rules are deliberately strict and conservative. A name that
//! passes `validate_name` is safe to use as a file name, a tree entry
//! name, a reference name, or a tag name without risking path traversal
//! or filesystem corruption.
//!
//! # Why these restrictions?
//!
//! - **No `/`** – prevents path injection. A name like `"../../etc/passwd"`
//!   could trick a naive backend into writing files outside the
//!   repository.
//! - **No `.` or `..`** – prevents ambiguity in directory traversal.
//!   These are special directory entries on all major operating systems.
//! - **Length limit** – prevents denial‑of‑service via memory exhaustion.
//!
//! # When to use
//!
//! Call `validate_name` whenever you have a raw string that will be
//! used as a name in any object. The constructors in `libvctrl_handler`
//! already call this function (or equivalent validation), so if you are
//! using those constructors you do not need to call it separately.
//!
//! This function is exposed for cases where you need to validate names
//! in custom code, or when building components that accept names as
//! raw strings before passing them to constructors.
//!
//! # Example
//!
//! ```rust
//! use libvctrl_core::validate::name::validate_name;
//!
//! // Valid names
//! assert!(validate_name("hello").is_ok());
//! assert!(validate_name("README.md").is_ok());
//! assert!(validate_name("refs-heads-main").is_ok()); // '/' is not allowed
//!
//! // Invalid names
//! assert!(validate_name("").is_err());                  // empty
//! assert!(validate_name(&"a".repeat(300)).is_err());   // too long
//! assert!(validate_name("src/main.rs").is_err());      // contains '/'
//! assert!(validate_name("..").is_err());               // is '..'
//! assert!(validate_name(".").is_err());                // is '.'
//! ```

use libvctrl_handler::{MAX_NAME_LENGTH, VctrlError};

/// Validates a name according to the fundamental contracts.
///
/// A valid name:
/// - Is not empty.
/// - Does not exceed `MAX_NAME_LENGTH` bytes.
/// - Does not contain the path separator `/`.
/// - Is not `.` or `..`.
///
/// This function is the single point of truth for name validation in
/// `libvctrl_core`. All higher-level modules (builders, stores) call it
/// before constructing objects that carry a name.
///
/// # Errors
/// Returns [`VctrlError::InvalidName`] with a descriptive message.
///
/// # Examples
/// ```
/// use libvctrl_core::validate::name::validate_name;
/// assert!(validate_name("hello").is_ok());
/// assert!(validate_name("").is_err());
/// assert!(validate_name(&"a".repeat(300)).is_err());
/// assert!(validate_name("src/main.rs").is_err());  // contains '/'
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
    // Reject path separators and relative path components.
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
