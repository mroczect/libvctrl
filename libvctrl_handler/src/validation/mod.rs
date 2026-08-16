//! Pure validation functions for names, references, and hashes.
//!
//! # Architecture
//! This module separates validation logic from data structure construction. By isolating
//! these checks into pure, standalone functions, we adhere to the "fail-fast" principle:
//! inputs are scrutinized before any memory allocation or state mutation occurs.
//!
//! # Design Rationale: Pure Functions vs. Constructors
//! While constructors like [`Hash::from_bytes`](crate::Hash::from_bytes) also perform validation,
//! extracting these checks into standalone functions allows consumers to validate raw,
//! unstructured data (e.g., from network streams or untrusted user input) before deciding
//! how to process it. This avoids partial commits of invalid data and makes the validation
//! logic trivially testable without constructing the full object.
//!
//! # Safety and Performance
//! These functions are entirely pure with no side effects. They operate on borrowed slices
//! (`&str`, `&[u8]`) and perform zero heap allocations. The compiler aggressively inlines
//! these checks when used within constructors, achieving zero-cost abstraction.
//!
//! # Examples
//! *Note: The following examples assume this crate is named `libvctrl_handler`.*
//!
//! ```
//! # use libvctrl_handler::validation::validate_name;
//! # use libvctrl_handler::VctrlError;
//! let valid_name = "feature_branch";
//! assert!(validate_name(valid_name).is_ok());
//!
//! let invalid_name = "";
//! assert!(matches!(validate_name(invalid_name), Err(VctrlError::InvalidName(_))));
//! ```

/// Hash validation utilities.
///
/// # Why this exists
/// Provides standalone validation for byte slices intended to be used as Git object hashes.
/// This ensures that data read from untrusted sources (like network packfiles) is the correct
/// length and format before attempting to construct a [`Hash`](crate::Hash) type, preventing
/// unbound allocations or cryptographic mismatches.
pub mod hash;

/// Name and reference validation utilities.
///
/// # Why this exists
/// Git has strict rules for naming references (branches, tags) and tree entries.
/// For example, names cannot contain control characters, cannot be empty, and cannot
/// contain certain path components like `..`. This module enforces these rules to prevent
/// filesystem traversal vulnerabilities and repository corruption.
pub mod name;

/// Re-export of [`validate_hash_bytes`](hash::validate_hash_bytes) for ergonomic top-level access.
///
/// Validates that a byte slice is the correct length to be a hash.
///
/// # Examples
///
/// ```
/// # use libvctrl_handler::validation::validate_hash_bytes;
/// let valid_hash = [0u8; 64];
/// assert!(validate_hash_bytes(&valid_hash).is_ok());
/// ```
pub use hash::validate_hash_bytes;

/// Re-exports of name and reference validation utilities.
///
/// Provides ergonomic access to functions that enforce Git naming rules.
///
/// # Examples
///
/// ```
/// # use libvctrl_handler::validation::{validate_name, validate_ref_name, validate_tree_entry_name};
/// assert!(validate_name("valid_name").is_ok());
/// assert!(validate_ref_name("refs/heads/main").is_ok());
/// assert!(validate_tree_entry_name("file.txt").is_ok());
/// ```
pub use name::{validate_name, validate_ref_name, validate_tree_entry_name};
