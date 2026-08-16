//! Validation functions for names, references, and hashes.

/// Hash validation.
pub mod hash;

/// Name and reference validation.
pub mod name;

pub use hash::validate_hash_bytes;
pub use name::{validate_name, validate_ref_name, validate_tree_entry_name};
