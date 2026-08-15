//! Validation module.

/// Hash validation.
pub mod hash;

/// Name validation.
pub mod name;

pub use name::{validate_name, validate_ref_name, validate_tree_entry_name};
