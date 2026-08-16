//! Name and reference validation utilities.
//!
//! # Architecture
//! Git has strict rules for naming references (branches, tags) and tree entries.
//! This module enforces these rules to prevent filesystem traversal vulnerabilities,
//! repository corruption, and ambiguity in revision parsing.
//!
//! # Design Rationale: Layered Validation
//! Validation is structured hierarchically. [`validate_name`] provides baseline
//! sanitization (length, emptiness, control characters). Specialized functions
//! like [`validate_ref_name`] and [`validate_tree_entry_name`] build upon this
//! baseline, adding domain-specific constraints. This prevents duplication and
//! ensures all names are fundamentally safe before context-specific rules are applied.

use crate::constants::MAX_NAME_LENGTH;
use crate::errors::VctrlError;
use std::path::Path;

/// Validates a generic name.
///
/// # Why this exists
/// Establishes the minimum safety criteria for any string used as an identifier
/// in the version control system. It prevents empty strings (which cause ambiguity),
/// excessively long strings (which can exhaust memory or trigger filesystem errors),
/// and ASCII control characters (which can corrupt terminal output or interprocess
/// communication).
///
/// # How it works
/// The function checks the byte length of the string against [`MAX_NAME_LENGTH`].
/// Because [`MAX_NAME_LENGTH`] is a `u64`, it must be safely downcast to `usize`
/// using `try_from` to support 32-bit architectures where `usize` is smaller than `u64`.
/// It then iterates over the bytes to detect ASCII control characters (e.g., `\0`, `\n`, `\t`).
///
/// # Errors
///
/// Returns [`VctrlError::InvalidName`] if the name is empty, exceeds the maximum
/// allowed length, or contains ASCII control characters.
///
/// # Examples
///
/// ```
/// # use libvctrl_handler::validation::validate_name;
/// assert!(validate_name("valid_name").is_ok());
/// assert!(validate_name("").is_err());
/// assert!(validate_name(&"a".repeat(256)).is_err());
/// assert!(validate_name("invalid\nname").is_err());
/// ```
pub fn validate_name(name: &str) -> Result<(), VctrlError> {
    if name.is_empty() {
        return Err(VctrlError::InvalidName("name is empty".into()));
    }
    let max_len = usize::try_from(MAX_NAME_LENGTH).unwrap_or(usize::MAX);
    if name.len() > max_len {
        return Err(VctrlError::InvalidName(format!(
            "name exceeds maximum length {MAX_NAME_LENGTH}: '{name}'"
        )));
    }
    if name.bytes().any(|b| b.is_ascii_control()) {
        return Err(VctrlError::InvalidName(format!(
            "name contains control characters: '{name}'"
        )));
    }
    Ok(())
}

/// Validates a reference name (e.g., branch or tag) strictly according to Git rules.
///
/// # Why this exists
/// Git references map directly to the filesystem (e.g., `.git/refs/heads/main`).
/// Without strict validation, a malicious reference name could traverse the filesystem
/// (e.g., `../../etc/passwd`) or create ambiguous revision queries (e.g., names
/// containing `..` or `~`). This function enforces the rules defined in
/// `git-check-ref-format`.
///
/// # How it works
/// It first applies baseline validation via [`validate_name`]. It then checks for
/// forbidden sequences:
/// - `..`: Prevents path traversal and ambiguous range specifiers.
/// - `~`, `^`, `:`: Prevents ambiguity with revision specifiers (e.g., `HEAD~1`).
/// - `.lock` extension: Prevents race conditions with Git's internal lock files.
/// - Leading/trailing dots or slashes: Prevents hidden files or directory confusion.
///
/// # Errors
///
/// Returns [`VctrlError::InvalidName`] if the name fails basic name validation
/// or contains forbidden characters or patterns.
///
/// # Examples
///
/// ```
/// # use libvctrl_handler::validation::validate_ref_name;
/// assert!(validate_ref_name("refs/heads/main").is_ok());
/// assert!(validate_ref_name("feature/branch").is_ok());
///
/// // Path traversal is forbidden
/// assert!(validate_ref_name("refs/heads/../danger").is_err());
///
/// // Cannot end with .lock
/// assert!(validate_ref_name("refs/heads/config.lock").is_err());
/// ```
pub fn validate_ref_name(name: &str) -> Result<(), VctrlError> {
    validate_name(name)?;
    if name.contains("..")
        || name.contains('~')
        || name.contains('^')
        || name.contains(':')
        || name.contains('?')
        || name.contains('*')
        || name.contains('[')
        || name.contains('\\')
        || name.contains(' ')
        || name.contains("@{")
        || name.contains("//")
        || name.starts_with('.')
        || name.starts_with('/')
        || name.ends_with('/')
        || name.ends_with('.')
        || name.contains('<')
        || name.contains('>')
        || name.contains('|')
        || name.contains('"')
        || Path::new(name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("lock"))
    {
        return Err(VctrlError::InvalidName(format!(
            "invalid ref name: '{name}'"
        )));
    }
    Ok(())
}

/// Validates a tree entry name strictly.
///
/// # Why this exists
/// A tree entry represents a single file or subdirectory. Its name must be a
/// single path component, not a full path. Allowing path separators (`/` or `\`)
/// or directory aliases (`.` or `..`) would corrupt the tree hierarchy by injecting
/// implicit directories or allowing traversal outside the tree.
///
/// # How it works
/// After baseline validation via [`validate_name`], it scans for `/` and `\`
/// characters and explicitly rejects the strings `.` and `..`.
///
/// # Errors
///
/// Returns [`VctrlError::InvalidName`] if the name fails basic name validation
/// or contains forbidden path characters or names.
///
/// # Examples
///
/// ```
/// # use libvctrl_handler::validation::validate_tree_entry_name;
/// assert!(validate_tree_entry_name("file.txt").is_ok());
/// assert!(validate_tree_entry_name("src").is_ok());
///
/// // Path separators are forbidden
/// assert!(validate_tree_entry_name("dir/file.txt").is_err());
/// assert!(validate_tree_entry_name("dir\\file.txt").is_err());
///
/// // Directory aliases are forbidden
/// assert!(validate_tree_entry_name(".").is_err());
/// assert!(validate_tree_entry_name("..").is_err());
/// ```
pub fn validate_tree_entry_name(name: &str) -> Result<(), VctrlError> {
    validate_name(name)?;
    if name.contains('/') || name.contains('\\') || name == "." || name == ".." {
        return Err(VctrlError::InvalidName(format!(
            "tree entry name contains forbidden path characters or names: '{name}'"
        )));
    }
    Ok(())
}
